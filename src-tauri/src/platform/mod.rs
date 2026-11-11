//! The single OS abstraction. `core` and `commands` only see `CaptureBackend`; each OS
//! implements it with native APIs (macOS: Swift `FrameLyKit`; Windows: Win32/WinRT).
//!
//! Methods are blocking (some wait on OS prompts or async system APIs) and must be called
//! off the main thread — the command layer uses `spawn_blocking`.
//!
//! Screenshot (Stage 2) and recording (Stages 3–5) methods join this trait with the
//! features that implement them.

use crate::core::errors::AppResult;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A connected monitor.
///
/// `bounds` is in the OS window-system's desktop space: points on macOS, physical pixels on
/// Windows (per-monitor DPI aware). `pixel_width`/`pixel_height` are always native pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    pub id: String,
    pub name: String,
    pub bounds: Rect,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub scale_factor: f64,
    pub refresh_rate: Option<f64>,
    pub is_primary: bool,
}

/// A capturable top-level application window (FrameLy's own windows are excluded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app_name: String,
    /// Bundle identifier (macOS) or executable path (Windows).
    pub app_id: Option<String>,
    pub pid: u32,
    pub bounds: Rect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    Screen,
    Microphone,
}

impl Permission {
    pub fn as_str(self) -> &'static str {
        match self {
            Permission::Screen => "screen",
            Permission::Microphone => "microphone",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionState {
    Granted,
    Denied,
    /// Never asked; requesting will show the OS prompt.
    NotDetermined,
    /// Blocked by policy (MDM / parental controls) or not supported on this system.
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub screen: PermissionState,
    pub microphone: PermissionState,
}

pub trait CaptureBackend: Send + Sync {
    fn displays(&self) -> AppResult<Vec<Display>>;
    /// Requires screen permission on macOS (window titles are protected).
    fn windows(&self) -> AppResult<Vec<WindowInfo>>;
    fn audio_inputs(&self) -> AppResult<Vec<AudioDevice>>;
    fn permissions(&self) -> Permissions;
    /// Shows the OS prompt when possible and returns the resulting state.
    fn request_permission(&self, p: Permission) -> AppResult<PermissionState>;
    /// Deep link into the OS privacy settings for `p`, if the OS has one.
    fn permission_settings_url(&self, p: Permission) -> Option<&'static str>;
}

pub fn backend() -> Box<dyn CaptureBackend> {
    #[cfg(target_os = "macos")]
    return Box::new(macos::MacBackend);
    #[cfg(windows)]
    return Box::new(windows::WinBackend);
    #[cfg(not(any(target_os = "macos", windows)))]
    compile_error!("FrameLy supports macOS and Windows only");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_for_the_ui() {
        let p = Permissions {
            screen: PermissionState::NotDetermined,
            microphone: PermissionState::Granted,
        };
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"screen":"notDetermined","microphone":"granted"}"#
        );
        let d = Display {
            id: "1".into(),
            name: "Built-in".into(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 1512.0,
                height: 982.0,
            },
            pixel_width: 3024,
            pixel_height: 1964,
            scale_factor: 2.0,
            refresh_rate: Some(120.0),
            is_primary: true,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["pixelWidth"], 3024);
        assert_eq!(v["isPrimary"], true);
    }

    /// Live smoke test against the real OS APIs: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_enumeration() {
        let b = backend();
        let displays = b.displays().unwrap();
        assert!(!displays.is_empty(), "at least one display");
        assert_eq!(
            displays.iter().filter(|d| d.is_primary).count(),
            1,
            "exactly one primary"
        );
        println!("displays: {displays:#?}");
        println!("audio: {:#?}", b.audio_inputs().unwrap());
        println!("permissions: {:?}", b.permissions());
        match b.windows() {
            Ok(w) => println!("{} windows, e.g. {:?}", w.len(), w.first()),
            Err(e) => println!("windows unavailable: {e}"),
        }
    }
}
