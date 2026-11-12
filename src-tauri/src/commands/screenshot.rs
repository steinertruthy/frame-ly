use crate::{
    core::errors::{AppError, AppResult},
    screenshot::{self, CaptureInfo, OverlayInit, Request},
    AppState,
};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn take_screenshot(app: AppHandle, request: Request) -> AppResult<()> {
    screenshot::start(app, request).await
}

#[tauri::command]
pub fn overlay_init(app: AppHandle, display_id: String) -> AppResult<OverlayInit> {
    screenshot::overlay_init(&app, &display_id)
}

/// A region chosen in an overlay, in that overlay's CSS pixels.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    viewport_width: f64,
    viewport_height: f64,
}

#[tauri::command]
pub async fn finish_region(app: AppHandle, display_id: String, selection: Selection) -> AppResult<()> {
    let Selection { x, y, width, height, viewport_width, viewport_height } = selection;
    screenshot::finish_region(app, display_id, (x, y, width, height), (viewport_width, viewport_height)).await
}

#[tauri::command]
pub async fn finish_window(app: AppHandle, window_id: String) -> AppResult<()> {
    screenshot::finish_window(app, window_id).await
}

#[tauri::command]
pub fn cancel_capture(app: AppHandle) {
    screenshot::cancel(&app);
}

fn last(state: &State<'_, AppState>) -> AppResult<std::sync::Arc<screenshot::LastCapture>> {
    state.screenshots.last.lock().unwrap().clone().ok_or_else(|| AppError::Runtime("no capture yet".into()))
}

#[tauri::command]
pub fn last_capture(state: State<'_, AppState>) -> Option<CaptureInfo> {
    state.screenshots.last.lock().unwrap().as_ref().map(|l| l.info())
}

#[tauri::command]
pub async fn copy_last_capture(state: State<'_, AppState>) -> AppResult<()> {
    let l = last(&state)?;
    tauri::async_runtime::spawn_blocking(move || crate::platform::clipboard::copy_image(&l.png, &l.frame))
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))?
}

#[tauri::command]
pub fn save_last_capture(app: AppHandle) -> AppResult<CaptureInfo> {
    screenshot::save_last(&app)
}

#[tauri::command]
pub fn reveal_last_capture(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let path = last(&state)?.path.clone().ok_or_else(|| AppError::Runtime("capture not saved".into()))?;
    app.opener().reveal_item_in_dir(path).map_err(|e| AppError::Platform(e.to_string()))
}

#[tauri::command]
pub fn open_last_capture(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let path = last(&state)?.path.clone().ok_or_else(|| AppError::Runtime("capture not saved".into()))?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::Platform(e.to_string()))
}

#[tauri::command]
pub fn close_preview(app: AppHandle) {
    if let Some(w) = app.get_webview_window(screenshot::PREVIEW) {
        let _ = w.destroy();
    }
}
