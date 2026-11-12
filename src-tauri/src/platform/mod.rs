//! The single OS abstraction. `core` and `commands` only see `CaptureBackend`; each OS
//! implements it with native APIs (macOS: Swift `FrameLyKit`; Windows: Win32/WinRT).
//!
//! Methods are blocking (some wait on OS prompts or async system APIs) and must be called
//! off the main thread — the command layer uses `spawn_blocking`.
//!
//! Recording is one-at-a-time per process; the native layer owns the media pipeline and the
//! app layer (`recording.rs`) owns state, files and UI.

use crate::core::{errors::AppResult, image::Frame};
use serde::{Deserialize, Serialize};

pub mod clipboard;
pub mod disk;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

/// Native window styling that Tauri doesn't expose (overlay levels, capture exclusion…).
#[cfg(target_os = "macos")]
pub use macos::chrome;
#[cfg(windows)]
pub use windows::chrome;

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
    /// Usable area (excludes menu bar / Dock / taskbar), same space as `bounds`.
    pub work_area: Rect,
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
    /// Full-resolution image of one display, without FrameLy's own windows where the OS allows.
    fn capture_display(&self, display_id: &str) -> AppResult<Frame>;
    /// One window's content, even if partially covered by other windows.
    fn capture_window(&self, window_id: &str) -> AppResult<Frame>;
    /// The display under the mouse pointer.
    fn display_at_cursor(&self) -> Option<String>;

    /// Whether this platform build can record video yet (Windows: Stage 4).
    fn supports_recording(&self) -> bool;
    /// Starts capturing and encoding to `config.output_path`. `events` receives native events
    /// (first frame, system-initiated stop) on an arbitrary thread.
    fn start_recording(&self, config: &NativeRecording, events: RecordingEvents) -> AppResult<()>;
    fn pause_recording(&self);
    fn resume_recording(&self);
    /// Stops and finalizes the file (blocking).
    fn stop_recording(&self) -> AppResult<RecordingStats>;
    /// Releases a recording the system already stopped and finalized.
    fn discard_recording(&self);
    /// Current peak levels (0…1) of the sources in the active recording.
    fn recording_levels(&self) -> AudioLevels;

    /// "Test microphone" meter, independent of recording.
    fn mic_monitor_start(&self, device_id: Option<&str>) -> AppResult<()>;
    fn mic_monitor_level(&self) -> f32;
    fn mic_monitor_stop(&self);

    /// Duration and size of a video file.
    fn video_info(&self, path: &std::path::Path) -> AppResult<crate::core::library::Meta>;
    /// JPEG thumbnail (longest side ≤ `max_pixels`) of an image or video at `dest`.
    fn thumbnail(&self, path: &std::path::Path, dest: &std::path::Path, max_pixels: u32) -> AppResult<()>;
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioLevels {
    pub system: f32,
    pub microphone: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeAudio {
    pub system: bool,
    pub microphone: bool,
    pub microphone_id: Option<String>,
    pub system_gain: f32,
    pub microphone_gain: f32,
}

pub type RecordingEvents = std::sync::Arc<dyn Fn(NativeEvent) + Send + Sync>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTarget {
    /// "display" | "window" | "region"
    pub kind: &'static str,
    pub display_id: Option<String>,
    pub window_id: Option<String>,
    /// Region in display-local OS units (points on macOS, physical px on Windows).
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRecording {
    pub target: NativeTarget,
    pub audio: Option<NativeAudio>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub bitrate: u32,
    pub codec: crate::core::video::Codec,
    pub shows_cursor: bool,
    pub output_path: String,
    pub thumbnail_path: String,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStats {
    pub duration_ms: u64,
    pub frames: u64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub audio: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NativeEvent {
    /// First frame written.
    Started,
    /// The OS ended the capture (window closed, display unplugged, user clicked the system
    /// "Stop" control, permission revoked). The file is already finalized.
    #[serde(rename_all = "camelCase")]
    Stopped { reason: String, duration_ms: u64, frames: u64 },
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
        let e: NativeEvent = serde_json::from_str(
            r#"{"kind":"stopped","reason":"window closed","durationMs":1200,"frames":30}"#,
        )
        .unwrap();
        assert_eq!(e, NativeEvent::Stopped { reason: "window closed".into(), duration_ms: 1200, frames: 30 });
        assert_eq!(
            serde_json::from_str::<NativeEvent>(r#"{"kind":"started"}"#).unwrap(),
            NativeEvent::Started
        );
        let p = Permissions { screen: PermissionState::NotDetermined, microphone: PermissionState::Granted };
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"screen":"notDetermined","microphone":"granted"}"#
        );
        let d = Display {
            id: "1".into(),
            name: "Built-in".into(),
            bounds: Rect { x: 0.0, y: 0.0, width: 1512.0, height: 982.0 },
            work_area: Rect { x: 0.0, y: 33.0, width: 1512.0, height: 870.0 },
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
        assert_eq!(displays.iter().filter(|d| d.is_primary).count(), 1, "exactly one primary");
        println!("displays: {displays:#?}");
        println!("audio: {:#?}", b.audio_inputs().unwrap());
        println!("permissions: {:?}", b.permissions());
        match b.windows() {
            Ok(w) => println!("{} windows, e.g. {:?}", w.len(), w.first()),
            Err(e) => println!("windows unavailable: {e}"),
        }
        println!("cursor on display {:?}", b.display_at_cursor());
        match b.capture_display(&displays[0].id) {
            Ok(f) => {
                assert_eq!((f.width, f.height), (displays[0].pixel_width, displays[0].pixel_height));
                println!("captured {f:?}");
            }
            Err(e) => println!("capture unavailable: {e}"),
        }
    }
}
