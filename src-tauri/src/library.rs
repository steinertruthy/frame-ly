//! Library service: owns the index database, keeps it in sync with the media folders,
//! generates cached thumbnails on demand and performs file operations for the UI.

use crate::{
    core::{
        errors::{AppError, AppResult},
        library::{self, FileEntry, Item, Kind, Library, Meta, Query, Stats},
    },
    platform::CaptureBackend,
    AppState,
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Emitter, Manager};

pub const CHANGED_EVENT: &str = "library://changed";
const THUMB_PX: u32 = 480;

#[derive(Default)]
pub struct LibraryService {
    db: Mutex<Option<Library>>,
    reconciling: AtomicBool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    #[serde(flatten)]
    pub stats: Stats,
    /// Free space on the recordings volume.
    pub free_bytes: Option<u64>,
}

fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

fn with_db<T>(app: &AppHandle, f: impl FnOnce(&mut Library) -> AppResult<T>) -> AppResult<T> {
    let st = state(app);
    let mut guard = st.library.db.lock().unwrap();
    let db = guard.as_mut().ok_or_else(|| AppError::Runtime("library not ready".into()))?;
    f(db)
}

fn dirs(app: &AppHandle) -> Vec<PathBuf> {
    let s = state(app).settings.get().storage;
    vec![s.recordings_dir, s.screenshots_dir]
}

/// Opens the index, lets the web views stream files from the media folders (and only those),
/// and starts a first sync in the background.
pub fn init(app: &AppHandle) -> AppResult<()> {
    let db = Library::open(&app.path().app_data_dir()?.join("library.sqlite3"))?;
    *state(app).library.db.lock().unwrap() = Some(db);
    allow_media_dirs(app);
    refresh(app);
    Ok(())
}

/// Asset protocol scope = the two media folders (video playback needs HTTP range support,
/// which the asset protocol provides).
pub fn allow_media_dirs(app: &AppHandle) {
    let scope = app.asset_protocol_scope();
    for dir in dirs(app) {
        if let Err(e) = scope.allow_directory(&dir, false) {
            tracing::warn!(dir = %dir.display(), error = %e, "asset scope");
        }
    }
}

fn probe(backend: &dyn CaptureBackend, f: &FileEntry) -> Meta {
    match f.kind {
        Kind::Image => {
            let dims = crate::core::image::image_dimensions(&f.path);
            Meta { width: dims.map(|d| d.0), height: dims.map(|d| d.1), duration_ms: None }
        }
        Kind::Video => backend.video_info(&f.path).unwrap_or_default(),
    }
}

/// Re-syncs the index with the folders in the background (coalesced if already running).
pub fn refresh(app: &AppHandle) {
    if state(app).library.reconciling.swap(true, Ordering::SeqCst) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let t = std::time::Instant::now();
        let files = library::scan(&dirs(&app));
        let backend = Arc::clone(&state(&app).backend);
        let result = with_db(&app, |db| db.reconcile(&files, &|f| probe(backend.as_ref(), f)));
        state(&app).library.reconciling.store(false, Ordering::SeqCst);
        match result {
            Ok(changes) => {
                tracing::info!(
                    files = files.len(),
                    ?changes,
                    ms = t.elapsed().as_millis() as u64,
                    "library synced"
                );
                if changes.any() {
                    let _ = app.emit(CHANGED_EVENT, ());
                }
            }
            Err(e) => tracing::warn!(error = %e, "library sync failed"),
        }
    });
}

/// Registers a file FrameLy just created (known metadata, no probing needed).
pub fn add(app: &AppHandle, path: &Path, meta: Meta) {
    match with_db(app, |db| db.upsert(path, meta)) {
        Ok(()) => {
            let _ = app.emit(CHANGED_EVENT, ());
        }
        Err(e) => tracing::warn!(error = %e, "could not index new file"),
    }
}

pub fn list(app: &AppHandle, q: &Query) -> AppResult<Vec<Item>> {
    with_db(app, |db| db.query(q))
}

pub fn overview(app: &AppHandle) -> AppResult<Overview> {
    let stats = with_db(app, |db| db.stats())?;
    let dir = state(app).settings.get().storage.recordings_dir;
    Ok(Overview { stats, free_bytes: crate::platform::disk::free_bytes(&dir) })
}

pub fn item(app: &AppHandle, id: i64) -> AppResult<Item> {
    with_db(app, |db| db.get(id))
}

pub fn rename(app: &AppHandle, id: i64, name: &str) -> AppResult<Item> {
    let item = with_db(app, |db| db.rename(id, name))?;
    let _ = app.emit(CHANGED_EVENT, ());
    Ok(item)
}

/// Moves the file to the Trash / Recycle Bin (recoverable) and drops it from the index.
pub fn trash(app: &AppHandle, id: i64) -> AppResult<()> {
    let item = item(app, id)?;
    trash::delete(&item.path).map_err(|e| AppError::Platform(format!("could not move to trash: {e}")))?;
    with_db(app, |db| db.remove(id))?;
    tracing::info!(path = %item.path, "moved to trash");
    let _ = app.emit(CHANGED_EVENT, ());
    Ok(())
}

pub fn copy(app: &AppHandle, id: i64) -> AppResult<()> {
    let item = item(app, id)?;
    match item.kind {
        Kind::Image => crate::platform::clipboard::copy_image_file(Path::new(&item.path)),
        Kind::Video => crate::platform::clipboard::copy_file(Path::new(&item.path)),
    }
}

/// JPEG thumbnail bytes for `framely://localhost/thumb/<id>`, generated once and cached
/// (the cache key includes the size so edited files get a fresh thumbnail).
pub fn thumbnail(app: &AppHandle, id: i64) -> AppResult<Vec<u8>> {
    let item = item(app, id)?;
    let dir = app.path().app_cache_dir()?.join("thumbs");
    std::fs::create_dir_all(&dir)?;
    let cached = dir.join(format!("lib-{}-{}.jpg", item.id, item.size));
    if !cached.exists() {
        state(app).backend.thumbnail(Path::new(&item.path), &cached, THUMB_PX)?;
    }
    Ok(std::fs::read(cached)?)
}
