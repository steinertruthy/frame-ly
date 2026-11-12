use crate::{
    core::errors::{AppError, AppResult},
    editor::{self, Source},
};
use tauri::{ipc::InvokeBody, AppHandle};

#[tauri::command]
pub fn editor_open(app: AppHandle, path: String) -> AppResult<()> {
    editor::open(&app, std::path::Path::new(&path))
}

#[tauri::command]
pub fn editor_source() -> AppResult<Source> {
    editor::info()
}

fn png_body(request: &tauri::ipc::Request<'_>) -> AppResult<Vec<u8>> {
    match request.body() {
        InvokeBody::Raw(bytes) => Ok(bytes.clone()),
        InvokeBody::Json(_) => Err(AppError::Runtime("expected raw PNG bytes".into())),
    }
}

/// Body: raw PNG bytes (sent as a Uint8Array, not JSON — screenshots are several MB).
#[tauri::command]
pub async fn editor_save(app: AppHandle, request: tauri::ipc::Request<'_>) -> AppResult<Source> {
    let png = png_body(&request)?;
    tauri::async_runtime::spawn_blocking(move || editor::save(&app, &png))
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))?
}

#[tauri::command]
pub async fn editor_copy(request: tauri::ipc::Request<'_>) -> AppResult<()> {
    let png = png_body(&request)?;
    tauri::async_runtime::spawn_blocking(move || editor::copy(png))
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))?
}
