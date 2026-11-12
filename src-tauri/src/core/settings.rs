//! Versioned, validated user settings persisted as JSON.
//!
//! Only settings that drive implemented features live here; new sections are added
//! together with the feature that uses them (missing fields fall back to defaults,
//! so older files keep loading). Structural changes bump `CURRENT_VERSION` and get
//! a step in `migrate`.

use super::{
    errors::{AppError, AppResult},
    video::{Codec, Quality, Resolution},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::RwLock,
};

pub const CURRENT_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "de")]
    De,
}

/// FrameLy's identity is dark; light and "follow the system" stay available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    #[default]
    Dark,
}

/// Interface animations. `System` honors the OS "reduce motion" preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    #[default]
    System,
    Full,
    Reduced,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct General {
    pub language: Language,
    pub launch_at_login: bool,
    /// Closing the window keeps FrameLy running in the menu bar / tray (otherwise it quits).
    pub close_to_tray: bool,
    /// Show the main window when FrameLy starts (off = start quietly in the menu bar / tray).
    pub show_on_launch: bool,
    /// System notifications when a capture or recording is saved without the preview card.
    pub notifications: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            language: Language::System,
            launch_at_login: false,
            close_to_tray: true,
            show_on_launch: true,
            notifications: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub theme: Theme,
    pub motion: Motion,
    /// The slowly moving blue light behind the interface.
    pub ambient_light: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self { theme: Theme::Dark, motion: Motion::System, ambient_light: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    /// Lossless, sharp text. Default.
    #[default]
    Png,
    /// Smaller files for photos and sharing.
    Jpeg,
}

/// Screenshot preferences. In the selection toolbar "Copy" always copies and "Save" always
/// saves; these choose what else happens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Capture {
    /// When saving, also copy to the clipboard.
    pub copy_to_clipboard: bool,
    /// When copying, also keep the file in the screenshots folder.
    pub save_to_disk: bool,
    /// Show the floating preview card after a capture.
    pub show_preview: bool,
    pub format: ImageFormat,
    /// JPEG quality, 50–100.
    pub jpeg_quality: u8,
}

impl Default for Capture {
    fn default() -> Self {
        Self {
            copy_to_clipboard: true,
            save_to_disk: true,
            show_preview: true,
            format: ImageFormat::Png,
            jpeg_quality: 90,
        }
    }
}

/// What the "Record" button records by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RecordTarget {
    #[default]
    Screen,
    Window,
    Region,
}

/// Video recording preferences. Defaults favor quality and compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Recording {
    pub fps: u32,
    pub quality: Quality,
    pub resolution: Resolution,
    pub codec: Codec,
    /// Seconds of countdown before recording starts (0 = none).
    pub countdown: u32,
    pub show_cursor: bool,
    pub target: RecordTarget,
    /// Show the floating preview card when a recording is saved.
    pub show_preview: bool,
}

impl Default for Recording {
    fn default() -> Self {
        Self {
            fps: 30,
            quality: Quality::High,
            resolution: Resolution::Native,
            codec: Codec::H264,
            countdown: 3,
            show_cursor: true,
            target: RecordTarget::Screen,
            show_preview: true,
        }
    }
}

/// Audio sources for recordings. System audio uses native capture (no virtual drivers).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Audio {
    pub system_audio: bool,
    /// Off by default: turning it on is when FrameLy asks for microphone permission.
    pub microphone: bool,
    /// Device id; `None` follows the system default input.
    pub microphone_device: Option<String>,
    /// Linear gains, 0.0–2.0.
    pub system_gain: f32,
    pub microphone_gain: f32,
}

impl Default for Audio {
    fn default() -> Self {
        Self {
            system_audio: true,
            microphone: false,
            microphone_device: None,
            system_gain: 1.0,
            microphone_gain: 1.0,
        }
    }
}

pub const MAX_GAIN: f32 = 2.0;

pub const ALLOWED_FPS: [u32; 3] = [24, 30, 60];
pub const MAX_COUNTDOWN: u32 = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    pub recordings_dir: PathBuf,
    pub screenshots_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    pub general: General,
    pub appearance: Appearance,
    pub capture: Capture,
    pub recording: Recording,
    pub audio: Audio,
    /// Global shortcuts (action → accelerator; empty = off).
    pub shortcuts: super::shortcuts::Shortcuts,
    pub storage: Storage,
}

/// OS-dependent default locations, injected so defaults are testable.
#[derive(Debug, Clone)]
pub struct DefaultDirs {
    pub videos: PathBuf,
    pub pictures: PathBuf,
}

impl DefaultDirs {
    /// `~/Movies/FrameLy` + `~/Pictures/FrameLy` on macOS; `Videos\FrameLy` + `Pictures\FrameLy` on Windows.
    pub fn from_os() -> Self {
        let home = dirs::home_dir().unwrap_or_else(std::env::temp_dir);
        Self {
            videos: dirs::video_dir().unwrap_or_else(|| home.join("Videos")).join("FrameLy"),
            pictures: dirs::picture_dir().unwrap_or_else(|| home.join("Pictures")).join("FrameLy"),
        }
    }
}

impl Settings {
    pub fn defaults(d: &DefaultDirs) -> Self {
        Self {
            version: CURRENT_VERSION,
            general: General::default(),
            appearance: Appearance::default(),
            capture: Capture::default(),
            recording: Recording::default(),
            audio: Audio::default(),
            shortcuts: super::shortcuts::defaults(),
            storage: Storage { recordings_dir: d.videos.clone(), screenshots_dir: d.pictures.clone() },
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        if !ALLOWED_FPS.contains(&self.recording.fps) {
            return Err(AppError::InvalidSettings(format!("fps must be one of {ALLOWED_FPS:?}")));
        }
        for (name, g) in
            [("systemGain", self.audio.system_gain), ("microphoneGain", self.audio.microphone_gain)]
        {
            if !(0.0..=MAX_GAIN).contains(&g) {
                return Err(AppError::InvalidSettings(format!("{name} must be between 0 and {MAX_GAIN}")));
            }
        }
        super::shortcuts::validate(&self.shortcuts, super::shortcuts::Os::current())?;
        if self.recording.countdown > MAX_COUNTDOWN {
            return Err(AppError::InvalidSettings(format!("countdown must be at most {MAX_COUNTDOWN}s")));
        }
        if !(50..=100).contains(&self.capture.jpeg_quality) {
            return Err(AppError::InvalidSettings("jpegQuality must be between 50 and 100".into()));
        }
        for (name, p) in [
            ("recordingsDir", &self.storage.recordings_dir),
            ("screenshotsDir", &self.storage.screenshots_dir),
        ] {
            if p.as_os_str().is_empty() || !p.is_absolute() {
                return Err(AppError::InvalidSettings(format!("{name} must be an absolute path")));
            }
        }
        Ok(())
    }

    /// Parses any stored version, upgrading it and filling missing fields with defaults.
    pub fn from_json(raw: &str, d: &DefaultDirs) -> AppResult<Self> {
        let mut value: Value = serde_json::from_str(raw)?;
        if !value.is_object() {
            return Err(AppError::InvalidSettings("root must be an object".into()));
        }
        migrate(&mut value);
        // Overlay stored values on top of the defaults so partial files load.
        let mut merged = serde_json::to_value(Settings::defaults(d))?;
        merge(&mut merged, value);
        let mut s: Settings = serde_json::from_value(merged)?;
        s.version = CURRENT_VERSION;
        Ok(s)
    }
}

fn migrate(value: &mut Value) {
    let version = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    if version < 1 {
        // v0 → v1: nothing structural yet; establishes the version field.
    }
    if version < 2 {
        // v1 → v2: the theme default became Dark, but people who already chose (or kept)
        // "system" keep it — write it explicitly so the new default doesn't override them.
        if value.get("appearance").and_then(|a| a.get("theme")).is_none() {
            value["appearance"]["theme"] = Value::from("system");
        }
    }
    value["version"] = Value::from(CURRENT_VERSION);
}

fn merge(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                match b.get_mut(&k) {
                    Some(slot) => merge(slot, v),
                    None => {
                        b.insert(k, v);
                    }
                }
            }
        }
        (slot, v) => *slot = v,
    }
}

/// Thread-safe settings holder bound to a file on disk.
pub struct SettingsStore {
    path: PathBuf,
    current: RwLock<Settings>,
}

impl SettingsStore {
    /// Loads `path`, or creates it with defaults. A corrupt file is moved aside
    /// (`settings.corrupt-<timestamp>.json`) instead of being silently overwritten.
    pub fn load(path: PathBuf, d: &DefaultDirs) -> AppResult<Self> {
        let settings = match fs::read_to_string(&path) {
            Ok(raw) => match Settings::from_json(&raw, d).and_then(|s| s.validate().map(|_| s)) {
                Ok(s) => s,
                Err(e) => {
                    let backup = path.with_file_name(format!(
                        "settings.corrupt-{}.json",
                        chrono::Local::now().format("%Y%m%d-%H%M%S")
                    ));
                    tracing::warn!(error = %e, backup = %backup.display(), "settings file unreadable; restoring defaults");
                    let _ = fs::rename(&path, &backup);
                    Settings::defaults(d)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Settings::defaults(d),
            Err(e) => return Err(e.into()),
        };
        write_atomic(&path, &settings)?;
        Ok(Self { path, current: RwLock::new(settings) })
    }

    pub fn get(&self) -> Settings {
        self.current.read().expect("settings lock poisoned").clone()
    }

    /// Validates, creates the storage folders, persists and swaps in `next`.
    pub fn replace(&self, mut next: Settings) -> AppResult<Settings> {
        next.version = CURRENT_VERSION;
        next.validate()?;
        fs::create_dir_all(&next.storage.recordings_dir)?;
        fs::create_dir_all(&next.storage.screenshots_dir)?;
        write_atomic(&self.path, &next)?;
        *self.current.write().expect("settings lock poisoned") = next.clone();
        Ok(next)
    }
}

/// Write to a sibling temp file then rename, so a crash never leaves a half-written file.
fn write_atomic(path: &Path, settings: &Settings) -> AppResult<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(settings)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs_in(root: &Path) -> DefaultDirs {
        DefaultDirs { videos: root.join("Videos/FrameLy"), pictures: root.join("Pictures/FrameLy") }
    }

    #[test]
    fn creates_defaults_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let path = tmp.path().join("cfg/settings.json");
        let store = SettingsStore::load(path.clone(), &d).unwrap();
        assert_eq!(store.get(), Settings::defaults(&d));
        assert!(path.exists());
    }

    #[test]
    fn roundtrips_through_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let path = tmp.path().join("settings.json");
        let store = SettingsStore::load(path.clone(), &d).unwrap();
        let mut s = store.get();
        s.general.language = Language::PtBr;
        s.appearance.theme = Theme::Dark;
        store.replace(s.clone()).unwrap();
        assert!(d.videos.is_dir() && d.pictures.is_dir());
        let reloaded = SettingsStore::load(path, &d).unwrap();
        assert_eq!(reloaded.get(), s);
    }

    #[test]
    fn partial_and_unversioned_files_get_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let s =
            Settings::from_json(r#"{ "appearance": { "theme": "light" }, "unknownKey": 1 }"#, &d).unwrap();
        assert_eq!(s.version, CURRENT_VERSION);
        assert_eq!(s.appearance.theme, Theme::Light);
        assert_eq!(s.general.language, Language::System);
        assert_eq!(s.storage.recordings_dir, d.videos);
    }

    #[test]
    fn corrupt_file_is_backed_up_and_reset() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let path = tmp.path().join("settings.json");
        fs::write(&path, "{ not json").unwrap();
        let store = SettingsStore::load(path, &d).unwrap();
        assert_eq!(store.get(), Settings::defaults(&d));
        let backups = fs::read_dir(tmp.path())
            .unwrap()
            .filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().starts_with("settings.corrupt-"))
            .count();
        assert_eq!(backups, 1);
    }

    #[test]
    fn rejects_relative_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let store = SettingsStore::load(tmp.path().join("s.json"), &d).unwrap();
        let mut s = store.get();
        s.storage.screenshots_dir = PathBuf::from("relative/dir");
        let err = store.replace(s).unwrap_err();
        assert_eq!(err.code(), "invalid_settings");
        assert_eq!(store.get(), Settings::defaults(&d), "state unchanged after a rejected update");
    }

    #[test]
    fn jpeg_quality_is_bounded() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let store = SettingsStore::load(tmp.path().join("s.json"), &d).unwrap();
        let mut s = store.get();
        s.capture.jpeg_quality = 30;
        assert_eq!(store.replace(s.clone()).unwrap_err().code(), "invalid_settings");
        s.capture.jpeg_quality = 80;
        s.capture.format = ImageFormat::Jpeg;
        assert_eq!(store.replace(s).unwrap().capture.format, ImageFormat::Jpeg);
    }

    #[test]
    fn v1_files_keep_their_theme_and_gain_new_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let s = Settings::from_json(r#"{"version":1,"general":{"language":"pt-BR"},"appearance":{}}"#, &d)
            .unwrap();
        assert_eq!(s.appearance.theme, Theme::System, "v1 users keep following the system");
        assert!(s.general.close_to_tray && s.general.show_on_launch && !s.general.launch_at_login);
        assert_eq!(s.recording.target, RecordTarget::Screen);
        // Fresh installs get the dark identity.
        assert_eq!(Settings::defaults(&d).appearance.theme, Theme::Dark);
        let lang: Language = serde_json::from_str(r#""de""#).unwrap();
        assert_eq!(lang, Language::De);
    }

    #[test]
    fn stage1_files_gain_capture_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let s = Settings::from_json(r#"{"version":1,"general":{"language":"en"}}"#, &d).unwrap();
        assert_eq!(s.capture, Capture::default());
    }

    #[test]
    fn recording_values_are_validated() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let store = SettingsStore::load(tmp.path().join("s.json"), &d).unwrap();
        let mut s = store.get();
        s.recording.fps = 25;
        assert_eq!(store.replace(s.clone()).unwrap_err().code(), "invalid_settings");
        s.recording.fps = 60;
        s.recording.countdown = 11;
        assert_eq!(store.replace(s.clone()).unwrap_err().code(), "invalid_settings");
        s.recording.countdown = 5;
        s.recording.resolution = Resolution::P1080;
        assert_eq!(store.replace(s).unwrap().recording.resolution, Resolution::P1080);
    }

    #[test]
    fn audio_defaults_and_gain_bounds() {
        let tmp = tempfile::tempdir().unwrap();
        let d = dirs_in(tmp.path());
        let store = SettingsStore::load(tmp.path().join("s.json"), &d).unwrap();
        let mut s = store.get();
        assert!(s.audio.system_audio && !s.audio.microphone);
        s.audio.microphone_gain = 2.5;
        assert_eq!(store.replace(s.clone()).unwrap_err().code(), "invalid_settings");
        s.audio.microphone_gain = 1.5;
        s.audio.microphone_device = Some("BuiltInMicrophoneDevice".into());
        assert_eq!(store.replace(s).unwrap().audio.microphone_gain, 1.5);
    }

    #[test]
    fn recording_enums_use_readable_json() {
        let v = serde_json::to_value(Recording::default()).unwrap();
        assert_eq!(v["resolution"], "native");
        assert_eq!(v["codec"], "h264");
        assert_eq!(serde_json::to_value(Resolution::P1080).unwrap(), "1080p");
    }

    #[test]
    fn language_serializes_as_bcp47() {
        assert_eq!(serde_json::to_string(&Language::PtBr).unwrap(), r#""pt-BR""#);
    }
}
