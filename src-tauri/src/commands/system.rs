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

#[tauri::command]
pub fn panel_resize(app: AppHandle, height: f64) {
    crate::panel::resize(&app, height);
}

/// Opens the main window, optionally on a page ("home" | "library" | "settings").
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

/// Whether the main window should appear at startup (false when launched at login or when
/// the user prefers starting quietly). Answered once; later opens always show the window.
#[tauri::command]
pub fn startup_should_show(state: State<'_, AppState>) -> bool {
    !state.start_hidden.swap(false, std::sync::atomic::Ordering::SeqCst)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheInfo {
    bytes: u64,
}

fn dir_size(dir: &std::path::Path) -> u64 {
    std::fs::read_dir(dir)
        .map(|it| {
            it.flatten()
                .map(|e| match e.metadata() {
                    Ok(m) if m.is_dir() => dir_size(&e.path()),
                    Ok(m) => m.len(),
                    Err(_) => 0,
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Space used by FrameLy's temporary files (thumbnails and leftovers).
#[tauri::command]
pub fn storage_cache(app: AppHandle) -> AppResult<CacheInfo> {
    use tauri::Manager;
    Ok(CacheInfo { bytes: dir_size(&app.path().app_cache_dir()?) })
}

/// Deletes regenerable temporary files: thumbnails, and recording leftovers that aren't part
/// of an active or recoverable recording. Never touches the user's media folders.
#[tauri::command]
pub fn storage_cleanup(app: AppHandle, state: State<'_, AppState>) -> AppResult<CacheInfo> {
    use tauri::Manager;
    let cache = app.path().app_cache_dir()?;
    let before = dir_size(&cache);
    let _ = std::fs::remove_dir_all(cache.join("thumbs"));
    if !crate::recording::status(&app).phase.is_active() {
        let dir = cache.join("recordings");
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                let manifest = p.with_extension("json");
                if p.extension().and_then(|x| x.to_str()) != Some("json") && !manifest.exists() {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
    }
    let freed = before.saturating_sub(dir_size(&cache));
    tracing::info!(freed, "temporary files cleaned");
    let _ = state;
    Ok(CacheInfo { bytes: freed })
}

/// Plays the screenshot cue so people hear what "Sounds" means right after turning it on.
#[tauri::command]
pub fn preview_sound() {
    crate::platform::sound::play_wav(crate::core::sounds::wav_bytes(crate::core::sounds::Cue::Shutter));
}
