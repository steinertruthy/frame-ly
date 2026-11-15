use crate::{
    core::{
        errors::{AppError, AppResult},
        settings::{DefaultDirs, Settings},
    },
    AppState,
};
use tauri::{AppHandle, Emitter, State};

pub const SETTINGS_CHANGED: &str = "settings://changed";

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings.get()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    let previous = state.settings.get();
    let saved = state.settings.replace(settings)?;
    tracing::info!(
        language = ?saved.general.language,
        theme = ?saved.appearance.theme,
        "settings updated"
    );
    if previous.appearance.theme != saved.appearance.theme {
        crate::windows::apply_theme(&app, saved.appearance.theme);
    }
    if previous.general.language != saved.general.language {
        crate::tray::refresh_labels(&app, saved.general.language);
    }
    if previous.general.launch_at_login != saved.general.launch_at_login {
        sync_autostart(&app, saved.general.launch_at_login);
    }
    if previous.storage != saved.storage {
        crate::library::allow_media_dirs(&app);
        crate::library::refresh(&app);
    }
    if previous.shortcuts != saved.shortcuts {
        crate::shortcuts::apply(&app, &saved.shortcuts, state.backend.supports_recording());
    }
    app.emit(SETTINGS_CHANGED, &saved)?;
    Ok(saved)
}

/// Restores the default global shortcuts (defaults live only in `core::shortcuts`).
#[tauri::command]
pub fn reset_shortcuts(app: AppHandle, state: State<'_, AppState>) -> AppResult<Settings> {
    let mut next = state.settings.get();
    next.shortcuts = crate::core::shortcuts::defaults();
    update_settings(app, state, next)
}

/// Restores the defaults of one Settings category ("general", "captures", "recording", "audio",
/// "intelligence", "shortcuts", "appearance", "storage") or of everything ("all").
/// Data is never touched: recordings, transcripts, downloaded models and saved API keys stay.
#[tauri::command]
pub fn reset_settings(app: AppHandle, state: State<'_, AppState>, scope: String) -> AppResult<Settings> {
    let defaults = Settings::defaults(&DefaultDirs::from_os());
    let next = reset_scope(state.settings.get(), defaults, &scope)?;
    tracing::info!(scope, "settings reset to defaults");
    update_settings(app, state, next)
}

fn reset_scope(mut s: Settings, d: Settings, scope: &str) -> AppResult<Settings> {
    match scope {
        "general" => s.general = d.general,
        "captures" => s.capture = d.capture,
        "recording" => s.recording = d.recording,
        "audio" => s.audio = d.audio,
        "intelligence" => s.intelligence = d.intelligence,
        "shortcuts" => s.shortcuts = d.shortcuts,
        "appearance" => s.appearance = d.appearance,
        "storage" => s.storage = d.storage,
        "all" => s = d,
        other => return Err(AppError::InvalidSettings(format!("unknown settings scope: {other}"))),
    }
    Ok(s)
}

/// Makes the OS login item match the setting (LaunchAgent on macOS, Run key on Windows).
pub fn sync_autostart(app: &AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let current = launcher.is_enabled().unwrap_or(false);
    if current == enabled {
        return;
    }
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    match result {
        Ok(()) => tracing::info!(enabled, "launch at login updated"),
        Err(e) => tracing::warn!(enabled, error = %e, "could not update launch at login"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dirs() -> DefaultDirs {
        DefaultDirs { videos: PathBuf::from("/v"), pictures: PathBuf::from("/p") }
    }

    #[test]
    fn resets_only_the_chosen_scope() {
        let d = Settings::defaults(&dirs());
        let mut s = d.clone();
        s.recording.countdown = d.recording.countdown + 2;
        s.intelligence.local_only = !d.intelligence.local_only;
        let r = reset_scope(s.clone(), d.clone(), "intelligence").unwrap();
        assert_eq!(r.intelligence, d.intelligence);
        assert_eq!(r.recording, s.recording);
        assert_eq!(reset_scope(s, d.clone(), "all").unwrap(), d);
        assert!(reset_scope(d.clone(), d, "nope").is_err());
    }
}
