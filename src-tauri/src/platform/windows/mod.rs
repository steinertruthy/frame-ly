//! Windows backend (Win32 + WinRT via the `windows` crate).
//!
//! - Displays: `EnumDisplayMonitors` + DisplayConfig friendly names + per-monitor DPI.
//! - Windows: `EnumWindows`, filtered like Alt-Tab (visible, not cloaked, not tool windows).
//! - Audio: Core Audio `IMMDeviceEnumerator` capture endpoints.
//! - Permissions: unpackaged desktop apps get no OS prompt; we read the privacy consent store
//!   and deep-link to Settings.

mod audio;
mod capture;
pub mod chrome;
mod displays;
mod enum_windows;
mod permissions;

use super::{AudioDevice, CaptureBackend, Display, Permission, PermissionState, Permissions, WindowInfo};
use crate::core::{
    errors::{AppError, AppResult},
    image::Frame,
};
use windows::Win32::Foundation::HWND;

pub struct WinBackend;

pub(crate) fn win_err(context: &str, e: windows::core::Error) -> AppError {
    AppError::Platform(format!("{context}: {e}"))
}

impl CaptureBackend for WinBackend {
    fn displays(&self) -> AppResult<Vec<Display>> {
        displays::list()
    }

    fn windows(&self) -> AppResult<Vec<WindowInfo>> {
        enum_windows::list()
    }

    fn audio_inputs(&self) -> AppResult<Vec<AudioDevice>> {
        audio::list_inputs()
    }

    fn permissions(&self) -> Permissions {
        Permissions { screen: permissions::screen(), microphone: permissions::microphone() }
    }

    fn request_permission(&self, p: Permission) -> AppResult<PermissionState> {
        // Windows has no runtime consent prompt for unpackaged apps: report the current state;
        // the UI then offers the Settings deep link when it is denied.
        Ok(match p {
            Permission::Screen => permissions::screen(),
            Permission::Microphone => permissions::microphone(),
        })
    }

    fn permission_settings_url(&self, p: Permission) -> Option<&'static str> {
        match p {
            Permission::Screen => None,
            Permission::Microphone => Some("ms-settings:privacy-microphone"),
        }
    }

    fn capture_display(&self, display_id: &str) -> AppResult<Frame> {
        let monitor = displays::monitor_by_id(display_id)
            .ok_or_else(|| AppError::Platform(format!("display {display_id} not found")))?;
        capture::grab(&capture::item_for_monitor(monitor)?)
    }

    fn capture_window(&self, window_id: &str) -> AppResult<Frame> {
        let raw = usize::from_str_radix(window_id.trim_start_matches("0x"), 16)
            .map_err(|_| AppError::Platform(format!("invalid window id {window_id:?}")))?;
        capture::grab(&capture::item_for_window(HWND(raw as *mut _))?)
    }

    fn display_at_cursor(&self) -> Option<String> {
        displays::at_cursor()
    }

    fn supports_recording(&self) -> bool {
        false
    }

    fn start_recording(&self, _: &super::NativeRecording, _: super::RecordingEvents) -> AppResult<()> {
        Err(AppError::Platform("screen recording is not available on Windows yet".into()))
    }

    fn pause_recording(&self) {}

    fn resume_recording(&self) {}

    fn stop_recording(&self) -> AppResult<super::RecordingStats> {
        Err(AppError::Platform("no recording in progress".into()))
    }

    fn discard_recording(&self) {}
}
