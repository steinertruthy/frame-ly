//! Capture-source discovery and OS permissions.

use crate::{
    core::errors::{AppError, AppResult},
    platform::{AudioDevice, CaptureBackend, Display, Permission, PermissionState, Permissions, WindowInfo},
    AppState,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

/// Backend calls block (OS prompts, async system APIs); keep them off the main/IPC threads.
async fn blocking<T: Send + 'static>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&dyn CaptureBackend) -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    let backend = Arc::clone(&state.backend);
    tauri::async_runtime::spawn_blocking(move || f(backend.as_ref()))
        .await
        .map_err(|e| AppError::Runtime(e.to_string()))?
}

#[tauri::command]
pub async fn list_displays(state: State<'_, AppState>) -> AppResult<Vec<Display>> {
    let displays = blocking(&state, |b| b.displays()).await?;
    tracing::debug!(count = displays.len(), names = ?displays.iter().map(|d| &d.name).collect::<Vec<_>>(), "displays listed");
    Ok(displays)
}

#[tauri::command]
pub async fn list_windows(state: State<'_, AppState>) -> AppResult<Vec<WindowInfo>> {
    blocking(&state, |b| b.windows()).await
}

#[tauri::command]
pub async fn list_audio_inputs(state: State<'_, AppState>) -> AppResult<Vec<AudioDevice>> {
    blocking(&state, |b| b.audio_inputs()).await
}

#[tauri::command]
pub async fn get_permissions(state: State<'_, AppState>) -> AppResult<Permissions> {
    blocking(&state, |b| Ok(b.permissions())).await
}

#[tauri::command]
pub async fn request_permission(
    state: State<'_, AppState>,
    permission: Permission,
) -> AppResult<PermissionState> {
    let result = blocking(&state, move |b| b.request_permission(permission)).await;
    tracing::info!(permission = permission.as_str(), result = ?result, "permission requested");
    result
}

#[tauri::command]
pub fn open_permission_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    permission: Permission,
) -> AppResult<()> {
    let url = state
        .backend
        .permission_settings_url(permission)
        .ok_or_else(|| AppError::Platform("no settings page for this permission".into()))?;
    app.opener().open_url(url, None::<&str>).map_err(|e| AppError::Platform(e.to_string()))
}

/// macOS applies a newly granted Screen Recording permission only after a relaunch.
#[tauri::command]
pub fn relaunch(app: AppHandle) {
    tracing::info!("relaunching to apply permissions");
    app.restart();
}
