use crate::{core::errors::AppResult, AppState};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    platform: &'static str,
    arch: &'static str,
    log_dir: String,
    supports_recording: bool,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>, app: AppHandle) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        platform: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        log_dir: state.log_dir.display().to_string(),
        supports_recording: state.backend.supports_recording(),
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Folder {
    Recordings,
    Screenshots,
    Logs,
}

/// Opens one of FrameLy's own folders in Finder / Explorer. Only these known folders are
/// allowed, so the web layer never gets a generic "open any path" capability.
#[tauri::command]
pub fn open_folder(app: AppHandle, state: State<'_, AppState>, folder: Folder) -> AppResult<()> {
    let settings = state.settings.get();
    let path = match folder {
        Folder::Recordings => settings.storage.recordings_dir,
        Folder::Screenshots => settings.storage.screenshots_dir,
        Folder::Logs => state.log_dir.clone(),
    };
    std::fs::create_dir_all(&path)?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| crate::core::errors::AppError::Runtime(e.to_string()))
}

/// Forwards UI errors into the structured log so problems in any window are diagnosable.
#[tauri::command]
pub fn log_ui(window: tauri::Window, level: String, message: String) {
    match level.as_str() {
        "error" => tracing::error!(window = window.label(), %message, "ui"),
        "warn" => tracing::warn!(window = window.label(), %message, "ui"),
        _ => tracing::info!(window = window.label(), %message, "ui"),
    }
}

#[tauri::command]
pub fn panel_hide(app: AppHandle) {
    crate::panel::hide(&app);
}

/// Opens the main window, optionally on a page ("home" | "settings").
#[tauri::command]
pub fn show_main_window(app: AppHandle, page: Option<String>) -> AppResult<()> {
    use tauri::Emitter;
    crate::panel::hide(&app);
    crate::windows::show_main(&app);
    if let Some(page) = page {
        app.emit_to(crate::windows::MAIN, "main://navigate", page)?;
    }
    Ok(())
}

#[tauri::command]
pub fn shortcut_status() -> crate::shortcuts::Status {
    crate::shortcuts::status()
}

/// Called while Settings records a new shortcut, so existing ones don't fire.
#[tauri::command]
pub fn suspend_shortcuts(app: AppHandle) {
    crate::shortcuts::suspend(&app);
}

#[tauri::command]
pub fn resume_shortcuts(app: AppHandle, state: State<'_, AppState>) {
    crate::shortcuts::apply(&app, &state.settings.get().shortcuts, state.backend.supports_recording());
}
