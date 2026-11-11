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
    app.emit(SETTINGS_CHANGED, &saved)?;
    Ok(saved)
}
