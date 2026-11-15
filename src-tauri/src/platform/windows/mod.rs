//! Windows backend (Win32 + WinRT via the `windows` crate).
//!
//! - Displays: `EnumDisplayMonitors` + DisplayConfig friendly names + per-monitor DPI.
//! - Windows: `EnumWindows`, filtered like Alt-Tab (visible, not cloaked, not tool windows).
//! - Audio: Core Audio `IMMDeviceEnumerator` capture endpoints.
//! - Permissions: unpackaged desktop apps get no OS prompt; we read the privacy consent store
//!   and deep-link to Settings.
//! - Recording: Windows.Graphics.Capture → D3D11 video processor → Media Foundation (hardware
//!   H.264/HEVC, AAC) with WASAPI system audio + microphone (see `recorder`).

mod audio;
mod capture;
pub mod chrome;
mod displays;
mod enum_windows;
pub mod keystore;
mod media;
pub use media::decode_audio;
mod permissions;
mod recorder;
mod wasapi;

use super::{AudioDevice, CaptureBackend, Display, Permission, PermissionState, Permissions, WindowInfo};
use crate::core::{
    errors::{AppError, AppResult},
    image::Frame,
};
use std::sync::Mutex;
use windows::Win32::Foundation::HWND;

pub struct WinBackend;

/// Settings' "test your microphone" meter (one at a time).
static MIC_TEST: Mutex<Option<wasapi::MicMonitor>> = Mutex::new(None);

fn is_video(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "mp4" | "mov" | "m4v"))
}

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
        true
    }

    fn start_recording(
        &self,
        config: &super::NativeRecording,
        events: super::RecordingEvents,
    ) -> AppResult<()> {
        recorder::start(config, events)
    }

    fn pause_recording(&self) {
        recorder::pause();
    }

    fn resume_recording(&self) {
        recorder::resume();
    }

    fn stop_recording(&self) -> AppResult<super::RecordingStats> {
        recorder::stop()
    }

    fn discard_recording(&self) {
        recorder::discard();
    }

    fn recording_levels(&self) -> super::AudioLevels {
        recorder::levels()
    }

    fn set_audio_muted(&self, system: bool, microphone: bool) {
        recorder::set_muted(system, microphone);
    }

    fn mic_monitor_start(&self, device_id: Option<&str>) -> AppResult<()> {
        let mut current = MIC_TEST.lock().unwrap();
        *current = None; // stop the previous one before opening the device again
        *current = Some(wasapi::MicMonitor::start(device_id)?);
        Ok(())
    }

    fn mic_monitor_level(&self) -> f32 {
        MIC_TEST.lock().unwrap().as_ref().map_or(0.0, |m| m.level())
    }

    fn mic_monitor_stop(&self) {
        *MIC_TEST.lock().unwrap() = None;
    }

    fn video_info(&self, path: &std::path::Path) -> AppResult<crate::core::library::Meta> {
        media::video_info(path)
    }

    fn thumbnail(&self, path: &std::path::Path, dest: &std::path::Path, max_pixels: u32) -> AppResult<()> {
        if is_video(path) {
            media::video_thumbnail(path, dest, max_pixels)
        } else {
            crate::core::image::thumbnail_file(path, dest, max_pixels)
        }
    }
}
