//! macOS backend: thin FFI over FrameLyKit (Swift). See `native/macos/FrameLyKit/Sources/FrameLyKit/FFI.swift`
//! for the JSON envelope protocol.

pub mod chrome;

use super::{AudioDevice, CaptureBackend, Display, Permission, PermissionState, Permissions, WindowInfo};
use crate::core::{
    errors::{AppError, AppResult},
    image::Frame,
};
use serde::{de::DeserializeOwned, Deserialize};
use std::ffi::{c_char, CStr};

extern "C" {
    fn fl_free_string(ptr: *mut c_char);
    fn fl_list_displays() -> *mut c_char;
    fn fl_list_windows() -> *mut c_char;
    fn fl_list_audio_inputs() -> *mut c_char;
    fn fl_permissions() -> *mut c_char;
    fn fl_request_screen() -> *mut c_char;
    fn fl_request_microphone() -> *mut c_char;
    fn fl_capture_display(id: u32, out: *mut *mut u8, w: *mut u32, h: *mut u32) -> *mut c_char;
    fn fl_capture_window(id: u32, out: *mut *mut u8, w: *mut u32, h: *mut u32) -> *mut c_char;
    fn fl_free_buffer(ptr: *mut u8);
    fn fl_display_at_cursor() -> u32;
}

type CaptureFn = unsafe extern "C" fn(u32, *mut *mut u8, *mut u32, *mut u32) -> *mut c_char;

/// Calls a FrameLyKit capture export and copies the BGRA buffer into a `Frame`.
fn capture(f: CaptureFn, id: &str) -> AppResult<Frame> {
    let id: u32 = id.parse().map_err(|_| AppError::Platform(format!("invalid id {id:?}")))?;
    let (mut data, mut w, mut h) = (std::ptr::null_mut(), 0u32, 0u32);
    // SAFETY: on success FrameLyKit fills a malloc'd buffer of exactly w*h*4 bytes that we
    // copy and release once; on failure it returns an owned error string and no buffer.
    unsafe {
        let err = f(id, &mut data, &mut w, &mut h);
        if !err.is_null() {
            let json = CStr::from_ptr(err).to_string_lossy().into_owned();
            fl_free_string(err);
            return decode::<()>(&json).and(Err(AppError::Platform("capture failed".into())));
        }
        if data.is_null() {
            return Err(AppError::Platform("capture returned no data".into()));
        }
        let len = w as usize * h as usize * 4;
        let bytes = std::slice::from_raw_parts(data, len).to_vec();
        fl_free_buffer(data);
        Frame::new(w, h, bytes)
    }
}

#[derive(Deserialize)]
struct FfiError {
    code: String,
    message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Envelope<T> {
    Ok(T),
    Err(FfiError),
}

/// Calls a FrameLyKit export, takes ownership of the returned string and decodes the envelope.
fn call<T: DeserializeOwned>(f: unsafe extern "C" fn() -> *mut c_char) -> AppResult<T> {
    // SAFETY: every FrameLyKit export returns a non-null strdup'ed C string that we own and
    // release exactly once with fl_free_string after copying it.
    let json = unsafe {
        let ptr = f();
        if ptr.is_null() {
            return Err(AppError::Platform("FrameLyKit returned null".into()));
        }
        let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        fl_free_string(ptr);
        s
    };
    decode(&json)
}

fn decode<T: DeserializeOwned>(json: &str) -> AppResult<T> {
    match serde_json::from_str::<Envelope<T>>(json)? {
        Envelope::Ok(v) => Ok(v),
        Envelope::Err(e) if e.code == "permission_denied" => {
            Err(AppError::PermissionDenied(match e.message.as_str() {
                "microphone" => "microphone",
                _ => "screen",
            }))
        }
        Envelope::Err(e) => Err(AppError::Platform(e.message)),
    }
}

pub struct MacBackend;

impl CaptureBackend for MacBackend {
    fn displays(&self) -> AppResult<Vec<Display>> {
        call(fl_list_displays)
    }

    fn windows(&self) -> AppResult<Vec<WindowInfo>> {
        call(fl_list_windows)
    }

    fn audio_inputs(&self) -> AppResult<Vec<AudioDevice>> {
        call(fl_list_audio_inputs)
    }

    fn permissions(&self) -> Permissions {
        call(fl_permissions).unwrap_or_else(|e| {
            tracing::error!(error = %e, "failed to read permissions");
            Permissions { screen: PermissionState::NotDetermined, microphone: PermissionState::NotDetermined }
        })
    }

    fn request_permission(&self, p: Permission) -> AppResult<PermissionState> {
        match p {
            Permission::Screen => call(fl_request_screen),
            Permission::Microphone => call(fl_request_microphone),
        }
    }

    fn permission_settings_url(&self, p: Permission) -> Option<&'static str> {
        Some(match p {
            Permission::Screen => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"
            }
            Permission::Microphone => {
                "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
            }
        })
    }

    fn capture_display(&self, display_id: &str) -> AppResult<Frame> {
        capture(fl_capture_display, display_id)
    }

    fn capture_window(&self, window_id: &str) -> AppResult<Frame> {
        capture(fl_capture_window, window_id)
    }

    fn display_at_cursor(&self) -> Option<String> {
        // SAFETY: pure query with no pointers.
        Some(unsafe { fl_display_at_cursor() }.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_envelopes() {
        let ok: Vec<AudioDevice> = decode(r#"{"ok":[{"id":"a","name":"Mic","isDefault":true}]}"#).unwrap();
        assert_eq!(ok[0].name, "Mic");
        let err = decode::<Vec<WindowInfo>>(r#"{"err":{"code":"permission_denied","message":"screen"}}"#)
            .unwrap_err();
        assert_eq!(err.code(), "permission_denied");
        let err = decode::<Permissions>(r#"{"err":{"code":"platform","message":"boom"}}"#).unwrap_err();
        assert_eq!(err.code(), "platform");
    }

    #[test]
    fn real_ffi_roundtrip() {
        // Exercises the linked Swift library; no permission needed for these two.
        let displays = MacBackend.displays().unwrap();
        assert!(displays.iter().any(|d| d.is_primary));
        MacBackend.audio_inputs().unwrap();
    }
}
