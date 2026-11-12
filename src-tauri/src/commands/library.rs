use crate::{
    core::{
        errors::{AppError, AppResult},
        library::{Item, Query},
    },
    library::{self, Overview},
};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| AppError::Runtime(e.to_string()))?
}

#[tauri::command]
pub async fn library_list(app: AppHandle, query: Query) -> AppResult<Vec<Item>> {
    blocking(move || library::list(&app, &query)).await
}

#[tauri::command]
pub async fn library_overview(app: AppHandle) -> AppResult<Overview> {
    blocking(move || library::overview(&app)).await
}

/// Re-syncs with the folders (files added/removed outside FrameLy).
#[tauri::command]
pub fn library_refresh(app: AppHandle) {
    library::refresh(&app);
}

#[tauri::command]
pub async fn library_rename(app: AppHandle, id: i64, name: String) -> AppResult<Item> {
    blocking(move || library::rename(&app, id, &name)).await
}

#[tauri::command]
pub async fn library_trash(app: AppHandle, id: i64) -> AppResult<()> {
    blocking(move || library::trash(&app, id)).await
}

#[tauri::command]
pub async fn library_copy(app: AppHandle, id: i64) -> AppResult<()> {
    blocking(move || library::copy(&app, id)).await
}

#[tauri::command]
pub fn library_reveal(app: AppHandle, id: i64) -> AppResult<()> {
    let item = library::item(&app, id)?;
    app.opener().reveal_item_in_dir(&item.path).map_err(|e| AppError::Platform(e.to_string()))
}

#[tauri::command]
pub fn library_open(app: AppHandle, id: i64) -> AppResult<()> {
    let item = library::item(&app, id)?;
    app.opener().open_path(item.path, None::<&str>).map_err(|e| AppError::Platform(e.to_string()))
}
