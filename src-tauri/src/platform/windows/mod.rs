//! Windows backend (Win32 + WinRT via the `windows` crate).
//!
//! - Displays: `EnumDisplayMonitors` + DisplayConfig friendly names + per-monitor DPI.
//! - Windows: `EnumWindows`, filtered like Alt-Tab (visible, not cloaked, not tool windows).
//! - Audio: Core Audio `IMMDeviceEnumerator` capture endpoints.
//! - Permissions: unpackaged desktop apps get no OS prompt; we read the privacy consent store
//!   and deep-link to Settings.

mod audio;
mod displays;
mod enum_windows;
mod permissions;

use super::{
    AudioDevice, CaptureBackend, Display, Permission, PermissionState, Permissions, WindowInfo,
};
use crate::core::errors::{AppError, AppResult};

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
        Permissions {
            screen: permissions::screen(),
            microphone: permissions::microphone(),
        }
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
}
