use crate::{
    core::errors::{AppError, AppResult},
    recording::{self, RecordingInfo, Request, Status},
    AppState,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn start_recording(app: AppHandle, request: Request) -> AppResult<()> {
    recording::start(app, request).await
}

#[tauri::command]
pub fn pause_recording(app: AppHandle) -> AppResult<()> {
    recording::pause(&app)
}

#[tauri::command]
pub fn resume_recording(app: AppHandle) -> AppResult<()> {
    recording::resume(&app)
}

#[tauri::command]
pub async fn stop_recording(app: AppHandle) -> AppResult<()> {
    recording::stop(app).await
}

#[tauri::command]
pub fn cancel_countdown(app: AppHandle) {
    recording::cancel_countdown(&app);
}

#[tauri::command]
pub fn recording_status(app: AppHandle) -> Status {
    recording::status(&app)
}

#[tauri::command]
pub fn recovered_recordings(app: AppHandle) -> Vec<String> {
    recording::take_recovered(&app)
}

fn last(state: &State<'_, AppState>) -> AppResult<Arc<recording::LastRecording>> {
    state.recordings.last.lock().unwrap().clone().ok_or_else(|| AppError::Runtime("no recording yet".into()))
}

#[tauri::command]
pub fn last_recording(state: State<'_, AppState>) -> Option<RecordingInfo> {
    state.recordings.last.lock().unwrap().as_ref().map(|l| l.info())
}

#[tauri::command]
pub fn reveal_last_recording(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    app.opener().reveal_item_in_dir(&last(&state)?.path).map_err(|e| AppError::Platform(e.to_string()))
}

#[tauri::command]
pub fn open_last_recording(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let path = last(&state)?.path.to_string_lossy().into_owned();
    app.opener().open_path(path, None::<&str>).map_err(|e| AppError::Platform(e.to_string()))
}

/// Puts the video file itself on the clipboard (paste into Finder, Slack, Mail…).
#[tauri::command]
pub fn copy_last_recording(state: State<'_, AppState>) -> AppResult<()> {
    crate::platform::clipboard::copy_file(&last(&state)?.path)
}
