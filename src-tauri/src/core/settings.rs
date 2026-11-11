//! Versioned, validated user settings persisted as JSON.
//!
//! Only settings that drive implemented features live here; new sections are added
//! together with the feature that uses them (missing fields fall back to defaults,
//! so older files keep loading). Structural changes bump `CURRENT_VERSION` and get
//! a step in `migrate`.

use super::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::RwLock,
};

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    #[serde(rename = "system")]
    System,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct General {
    pub language: Language,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub theme: Theme,
}

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
            videos: dirs::video_dir()
                .unwrap_or_else(|| home.join("Videos"))
                .join("FrameLy"),
            pictures: dirs::picture_dir()
                .unwrap_or_else(|| home.join("Pictures"))
                .join("FrameLy"),
        }
    }
}

impl Settings {
    pub fn defaults(d: &DefaultDirs) -> Self {
        Self {
            version: CURRENT_VERSION,
            general: General::default(),
            appearance: Appearance::default(),
            storage: Storage {
                recordings_dir: d.videos.clone(),
                screenshots_dir: d.pictures.clone(),
            },
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        for (name, p) in [
            ("recordingsDir", &self.storage.recordings_dir),
            ("screenshotsDir", &self.storage.screenshots_dir),
        ] {
            if p.as_os_str().is_empty() || !p.is_absolute() {
                return Err(AppError::InvalidSettings(format!(
                    "{name} must be an absolute path"
                )));
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
        Ok(Self {
            path,
            current: RwLock::new(settings),
        })
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
        DefaultDirs {
            videos: root.join("Videos/FrameLy"),
            pictures: root.join("Pictures/FrameLy"),
        }
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
        let s = Settings::from_json(
            r#"{ "appearance": { "theme": "light" }, "unknownKey": 1 }"#,
            &d,
        )
        .unwrap();
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
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("settings.corrupt-")
            })
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
        assert_eq!(
            store.get(),
            Settings::defaults(&d),
            "state unchanged after a rejected update"
        );
    }

    #[test]
    fn language_serializes_as_bcp47() {
        assert_eq!(
            serde_json::to_string(&Language::PtBr).unwrap(),
            r#""pt-BR""#
        );
    }
}
