use crate::{
    core::{errors::AppResult, settings::Settings},
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
