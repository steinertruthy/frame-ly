//! Screenshot editor window. The editor only ever touches the one image it was opened with:
//! the file must live in FrameLy's media folders, it's served to the web view through
//! `framely://localhost/edit-source` (CORS-enabled, so the canvas can export), and results
//! come back as raw PNG bytes.

use crate::{
    core::{
        errors::{AppError, AppResult},
        library::{Kind, Meta},
        paths,
    },
    AppState,
};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const EDITOR: &str = "editor";
pub const LOAD_EVENT: &str = "editor://load";

static SOURCE: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub path: String,
    pub name: String,
}

fn source() -> AppResult<PathBuf> {
    SOURCE.lock().unwrap().clone().ok_or_else(|| AppError::Runtime("no image open in the editor".into()))
}

/// Only images inside the recordings/screenshots folders can be edited.
fn allowed(app: &AppHandle, path: &Path) -> AppResult<PathBuf> {
    let path = path.canonicalize().map_err(|_| AppError::NotFound)?;
    if Kind::of(&path) != Some(Kind::Image) {
        return Err(AppError::InvalidName("only images can be edited".into()));
    }
    let storage = app.state::<AppState>().settings.get().storage;
    let inside = [storage.screenshots_dir, storage.recordings_dir]
        .iter()
        .filter_map(|d| d.canonicalize().ok())
        .any(|d| path.starts_with(d));
    if !inside {
        return Err(AppError::Runtime("file is outside FrameLy's folders".into()));
    }
    Ok(path)
}

pub fn open(app: &AppHandle, path: &Path) -> AppResult<()> {
    let path = allowed(app, path)?;
    tracing::info!(path = %path.display(), "editor opened");
    *SOURCE.lock().unwrap() = Some(path);
    crate::panel::hide(app);
    if let Some(w) = app.get_webview_window(EDITOR) {
        app.emit_to(EDITOR, LOAD_EVENT, info()?)?;
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let theme = app.state::<AppState>().settings.get().appearance.theme;
    let builder = WebviewWindowBuilder::new(app, EDITOR, WebviewUrl::App("editor.html".into()))
        .title("FrameLy")
        .inner_size(1180.0, 780.0)
        .min_inner_size(900.0, 600.0);
    crate::windows::app_window(builder, theme)?;
    Ok(())
}

pub fn info() -> AppResult<Source> {
    let path = source()?;
    Ok(Source {
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: path.display().to_string(),
    })
}

/// Bytes of the image being edited (protocol route).
pub fn source_bytes() -> Option<Vec<u8>> {
    std::fs::read(source().ok()?).ok()
}

fn edited_name(original: &Path) -> String {
    let stem =
        original.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "FrameLy".into());
    // Re-editing "x (edited).png" produces "x (edited 2).png" via unique_path, not "x (edited) (edited)".
    let base = stem.strip_suffix(" (edited)").unwrap_or(&stem);
    format!("{base} (edited).png")
}

/// Saves the edited PNG next to the original (never overwriting it) and indexes it.
pub fn save(app: &AppHandle, png: &[u8]) -> AppResult<Source> {
    let original = source()?;
    let dims = png_dimensions(png)?;
    let dir = original.parent().map(Path::to_path_buf).ok_or(AppError::NotFound)?;
    let dest = paths::unique_path(&dir, &edited_name(&original));
    std::fs::write(&dest, png)?;
    tracing::info!(path = %dest.display(), width = dims.0, height = dims.1, "edited image saved");
    crate::library::add(app, &dest, Meta { width: Some(dims.0), height: Some(dims.1), duration_ms: None });
    Ok(Source {
        name: dest.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: dest.display().to_string(),
    })
}

pub fn copy(png: Vec<u8>) -> AppResult<()> {
    png_dimensions(&png)?;
    crate::platform::clipboard::copy_image_bytes(png)
}

/// Validates that the payload really is a PNG and returns its size.
fn png_dimensions(bytes: &[u8]) -> AppResult<(u32, u32)> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .map_err(|e| AppError::Platform(format!("invalid PNG from editor: {e}")))?;
    let info = reader.info();
    Ok((info.width, info.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edited_names_do_not_stack() {
        assert_eq!(
            edited_name(Path::new("/x/FrameLy 2026-11-11 at 21.03.01.png")),
            "FrameLy 2026-11-11 at 21.03.01 (edited).png"
        );
        assert_eq!(edited_name(Path::new("/x/shot (edited).png")), "shot (edited).png");
        assert_eq!(edited_name(Path::new("/x/photo.jpg")), "photo (edited).png");
    }

    #[test]
    fn rejects_non_png_payloads() {
        assert!(png_dimensions(b"not a png").is_err());
        let frame = crate::core::image::Frame::new(3, 2, vec![0; 24]).unwrap();
        let png = crate::core::image::encode_png(&frame).unwrap();
        assert_eq!(png_dimensions(&png).unwrap(), (3, 2));
    }
}
