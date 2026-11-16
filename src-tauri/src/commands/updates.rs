use crate::core::errors::AppResult;
use crate::updates::{self, Status};
use tauri::AppHandle;

#[tauri::command]
pub fn updates_status(app: AppHandle) -> Status {
    updates::status(&app)
}

/// Manual "Check for updates".
#[tauri::command]
pub async fn updates_check(app: AppHandle) -> Status {
    updates::check(&app, true).await
}

/// "Update and restart": downloads, waits for any recording to end, installs, restarts.
#[tauri::command]
pub async fn updates_install(app: AppHandle) -> AppResult<()> {
    updates::install(app).await
}

/// "Later": hides the notice until the next launch.
#[tauri::command]
pub fn updates_dismiss(app: AppHandle) {
    updates::dismiss(&app);
}
