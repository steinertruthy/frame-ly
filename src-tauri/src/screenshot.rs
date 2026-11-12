//! Screenshot flow. Lives above `platform` (pixels) and `core::image` (encoding) and owns the
//! interactive part: frozen frames, selection overlays, output and the preview window.
//!
//! Region/window flow:
//! 1. `start` captures every display at once (the "frozen" moment the user selects from),
//!    then opens one borderless overlay window per display.
//! 2. Each overlay loads its frame through the `framely://` protocol (`/frame/<display id>`).
//! 3. The overlay calls `finish_region` (crop of the frozen frame) or `finish_window`
//!    (native capture of that window, unobstructed), or `cancel`.
//! 4. `deliver` encodes PNG, saves and/or copies it, and shows the preview.

use crate::{
    core::{
        errors::{AppError, AppResult},
        image::{self, Frame},
        paths,
    },
    platform::{chrome, clipboard, CaptureBackend, Display, WindowInfo},
    AppState,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const OVERLAY_PREFIX: &str = "overlay-";
pub const PREVIEW: &str = "preview";
pub const COMPLETED_EVENT: &str = "capture://completed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectMode {
    Region,
    Window,
}

/// What to capture when the user asks for a screenshot.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Request {
    /// A whole display; `None` = the display under the mouse pointer.
    Display { display_id: Option<String> },
    /// Interactive selection with the overlay.
    Select { mode: SelectMode },
}

/// Why the selection overlay is open: the same picker serves screenshots and recordings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Purpose {
    Screenshot,
    Record,
}

struct Session {
    purpose: Purpose,
    mode: SelectMode,
    displays: Vec<Display>,
    frames: HashMap<String, Arc<Frame>>,
    windows: Vec<WindowInfo>,
}

/// Result of the most recent capture, kept for the preview window and its actions.
pub struct LastCapture {
    pub png: Arc<Vec<u8>>,
    pub frame: Arc<Frame>,
    pub width: u32,
    pub height: u32,
    pub path: Option<PathBuf>,
    pub copied: bool,
}

#[derive(Default)]
pub struct Screenshots {
    session: Mutex<Option<Session>>,
    pub last: Mutex<Option<Arc<LastCapture>>>,
}

/// Data an overlay needs to draw: its display, the windows on it, and the starting mode.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayInit {
    purpose: Purpose,
    mode: SelectMode,
    display: Display,
    frame_width: u32,
    frame_height: u32,
    windows: Vec<WindowInfo>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInfo {
    pub width: u32,
    pub height: u32,
    pub path: Option<String>,
    pub file_name: Option<String>,
    pub copied: bool,
}

impl LastCapture {
    pub fn info(&self) -> CaptureInfo {
        CaptureInfo {
            width: self.width,
            height: self.height,
            path: self.path.as_ref().map(|p| p.display().to_string()),
            file_name: self
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned()),
            copied: self.copied,
        }
    }
}

fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

async fn on_worker<T: Send + 'static>(f: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| AppError::Runtime(e.to_string()))?
}

pub async fn start(app: AppHandle, request: Request) -> AppResult<()> {
    let backend: Arc<dyn CaptureBackend> = Arc::clone(&state(&app).backend);
    match request {
        Request::Display { display_id } => {
            let t = Instant::now();
            let frame = on_worker(move || {
                let id = display_id
                    .or_else(|| backend.display_at_cursor())
                    .ok_or_else(|| AppError::Platform("no display found".into()))?;
                backend.capture_display(&id)
            })
            .await?;
            tracing::info!(ms = t.elapsed().as_millis() as u64, ?frame, "display captured");
            deliver(&app, frame, None, Intent::Auto).await
        }
        Request::Select { mode } => select(app, mode, Purpose::Screenshot).await,
    }
}

/// Freezes all displays and opens the selection overlays.
pub async fn select(app: AppHandle, mode: SelectMode, purpose: Purpose) -> AppResult<()> {
    let backend: Arc<dyn CaptureBackend> = Arc::clone(&state(&app).backend);
    {
        if state(&app).screenshots.session.lock().unwrap().is_some() {
            focus_overlays(&app);
            return Ok(());
        }
        let t = Instant::now();
        let (displays, frames, windows) = on_worker(move || {
            let displays = backend.displays()?;
            // Freeze every display at the same moment, in parallel.
            let frames = std::thread::scope(|s| {
                let jobs: Vec<_> = displays
                    .iter()
                    .map(|d| {
                        let b = &backend;
                        s.spawn(move || (d.id.clone(), b.capture_display(&d.id)))
                    })
                    .collect();
                jobs.into_iter().map(|j| j.join().expect("capture thread panicked")).collect::<Vec<_>>()
            });
            let mut map = HashMap::new();
            for (id, frame) in frames {
                map.insert(id, Arc::new(frame?));
            }
            let windows = backend.windows().unwrap_or_default();
            Ok((displays, map, windows))
        })
        .await?;
        tracing::info!(
            ms = t.elapsed().as_millis() as u64,
            displays = displays.len(),
            "displays frozen for selection"
        );

        *state(&app).screenshots.session.lock().unwrap() =
            Some(Session { purpose, mode, displays: displays.clone(), frames, windows });
        for (i, d) in displays.iter().enumerate() {
            if let Err(e) = open_overlay(&app, i, d) {
                cancel(&app);
                return Err(e);
            }
        }
        Ok(())
    }
}

fn open_overlay(app: &AppHandle, index: usize, display: &Display) -> AppResult<()> {
    let id: String =
        percent_encoding::utf8_percent_encode(&display.id, percent_encoding::NON_ALPHANUMERIC).to_string();
    let url = WebviewUrl::App(format!("overlay.html?d={id}").into());
    let b = display.bounds;
    let builder = WebviewWindowBuilder::new(app, format!("{OVERLAY_PREFIX}{index}"), url)
        .title("FrameLy")
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .visible(false)
        .focused(true)
        .visible_on_all_workspaces(true);
    #[cfg(target_os = "macos")]
    let builder = builder.position(b.x, b.y).inner_size(b.width, b.height);
    #[cfg(windows)]
    let builder = builder
        .position(b.x / display.scale_factor, b.y / display.scale_factor)
        .inner_size(b.width / display.scale_factor, b.height / display.scale_factor);
    let window = builder.build()?;
    chrome::configure_overlay(&window, display);
    Ok(())
}

fn overlays(app: &AppHandle) -> Vec<tauri::WebviewWindow> {
    app.webview_windows().into_iter().filter(|(l, _)| l.starts_with(OVERLAY_PREFIX)).map(|(_, w)| w).collect()
}

fn focus_overlays(app: &AppHandle) {
    if let Some(w) = overlays(app).first() {
        let _ = w.set_focus();
    }
}

fn close_overlays(app: &AppHandle) {
    for w in overlays(app) {
        let _ = w.destroy();
    }
}

pub fn cancel(app: &AppHandle) {
    close_overlays(app);
    if state(app).screenshots.session.lock().unwrap().take().is_some() {
        tracing::info!("capture cancelled");
    }
}

pub fn overlay_init(app: &AppHandle, display_id: &str) -> AppResult<OverlayInit> {
    let st = state(app);
    let guard = st.screenshots.session.lock().unwrap();
    let s = guard.as_ref().ok_or_else(|| AppError::Runtime("no capture in progress".into()))?;
    let display = s
        .displays
        .iter()
        .find(|d| d.id == display_id)
        .cloned()
        .ok_or_else(|| AppError::Runtime(format!("unknown display {display_id}")))?;
    let frame = &s.frames[display_id];
    let db = display.bounds;
    let windows = s
        .windows
        .iter()
        .filter(|w| {
            let b = w.bounds;
            b.x < db.x + db.width && b.x + b.width > db.x && b.y < db.y + db.height && b.y + b.height > db.y
        })
        .cloned()
        .collect();
    Ok(OverlayInit {
        purpose: s.purpose,
        mode: s.mode,
        frame_width: frame.width,
        frame_height: frame.height,
        display,
        windows,
    })
}

/// BMP bytes of a frozen frame for the `framely://frame/<id>` protocol route.
pub fn frozen_frame_bmp(app: &AppHandle, display_id: &str) -> Option<Vec<u8>> {
    let st = state(app);
    let frame = st.screenshots.session.lock().unwrap().as_ref()?.frames.get(display_id).cloned()?;
    Some(image::encode_bmp(&frame))
}

pub async fn finish_region(
    app: AppHandle,
    display_id: String,
    selection: (f64, f64, f64, f64),
    viewport: (f64, f64),
) -> AppResult<()> {
    let session = state(&app)
        .screenshots
        .session
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AppError::Runtime("no capture in progress".into()))?;
    close_overlays(&app);
    if session.purpose == Purpose::Record {
        let display = session
            .displays
            .into_iter()
            .find(|d| d.id == display_id)
            .ok_or_else(|| AppError::Runtime(format!("unknown display {display_id}")))?;
        // CSS px → the OS units of `display.bounds` (points / physical px), display-local.
        let k = display.bounds.width / viewport.0.max(1.0);
        let (x, y, w, h) = selection;
        let rect = crate::platform::Rect { x: x * k, y: y * k, width: w * k, height: h * k };
        return crate::recording::begin(app, crate::recording::Target::Region { display, rect }).await;
    }
    let frame = session
        .frames
        .get(&display_id)
        .cloned()
        .ok_or_else(|| AppError::Runtime(format!("unknown display {display_id}")))?;
    let rect = image::css_to_pixels(selection, viewport, (frame.width, frame.height));
    let cropped = on_worker(move || frame.crop(rect)).await?;
    deliver(&app, cropped, None, Intent::Auto).await
}

pub async fn finish_window(app: AppHandle, window_id: String) -> AppResult<()> {
    let session = state(&app)
        .screenshots
        .session
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AppError::Runtime("no capture in progress".into()))?;
    close_overlays(&app);
    if session.purpose == Purpose::Record {
        let window = session
            .windows
            .iter()
            .find(|w| w.id == window_id)
            .cloned()
            .ok_or_else(|| AppError::Runtime(format!("unknown window {window_id}")))?;
        let display = display_containing(&session.displays, &window).cloned();
        return crate::recording::begin(app, crate::recording::Target::Window { window, display }).await;
    }
    let backend = Arc::clone(&state(&app).backend);
    let frame = on_worker(move || match backend.capture_window(&window_id) {
        Ok(f) => Ok(f),
        Err(e) => {
            // E.g. the window closed meanwhile: fall back to its area on the frozen frame.
            tracing::warn!(error = %e, "native window capture failed; cropping frozen frame");
            crop_window_from_session(&session, &window_id).ok_or(e)
        }
    })
    .await?;
    deliver(&app, frame, None, Intent::Auto).await
}

/// The display holding the center of `w`.
pub fn display_containing<'a>(displays: &'a [Display], w: &WindowInfo) -> Option<&'a Display> {
    let (cx, cy) = (w.bounds.x + w.bounds.width / 2.0, w.bounds.y + w.bounds.height / 2.0);
    displays.iter().find(|d| {
        let b = d.bounds;
        cx >= b.x && cx < b.x + b.width && cy >= b.y && cy < b.y + b.height
    })
}

fn crop_window_from_session(s: &Session, window_id: &str) -> Option<Frame> {
    let w = s.windows.iter().find(|w| w.id == window_id)?;
    let d = display_containing(&s.displays, w)?;
    let frame = s.frames.get(&d.id)?;
    let sel = (w.bounds.x - d.bounds.x, w.bounds.y - d.bounds.y, w.bounds.width, w.bounds.height);
    frame.crop(image::css_to_pixels(sel, (d.bounds.width, d.bounds.height), (frame.width, frame.height))).ok()
}

/// What the user asked for. The selection toolbar sends Copy or Save; one-step captures
/// (full-screen shortcut, tray, CLI) use Auto and follow the settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Intent {
    Copy,
    Save,
    Auto,
}

/// (save?, copy?) for an intent: Copy/Save always do their thing; settings decide the extra.
pub fn plan(intent: Intent, copy_too: bool, save_too: bool) -> (bool, bool) {
    match intent {
        Intent::Save => (true, copy_too),
        Intent::Copy => (save_too, true),
        // Never lose a one-step capture: with both options off it is still saved.
        Intent::Auto => (save_too || !copy_too, copy_too),
    }
}

/// File bytes and extension in the user's chosen format.
fn encode_for_disk(
    frame: &Frame,
    png: &[u8],
    opts: &crate::core::settings::Capture,
) -> AppResult<(Vec<u8>, &'static str)> {
    use crate::core::settings::ImageFormat;
    Ok(match opts.format {
        ImageFormat::Png => (png.to_vec(), "png"),
        ImageFormat::Jpeg => (image::encode_jpeg(frame, opts.jpeg_quality)?, "jpg"),
    })
}

fn write_capture(dir: &std::path::Path, bytes: &[u8], ext: &str) -> AppResult<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = paths::unique_path(dir, &paths::capture_file_name(&chrono::Local::now(), ext));
    std::fs::write(&path, bytes)?;
    Ok(path)
}

/// Encodes, saves and/or copies the capture, then shows the preview (or a notification).
/// `png` may be passed when already encoded (annotated captures come in as PNG).
pub async fn deliver(app: &AppHandle, frame: Frame, png: Option<Vec<u8>>, intent: Intent) -> AppResult<()> {
    crate::notify::cue(app, crate::core::sounds::Cue::Shutter);
    let settings = state(app).settings.get();
    let dir = settings.storage.screenshots_dir.clone();
    let opts = settings.capture.clone();
    let (save, copy) = plan(intent, opts.copy_to_clipboard, opts.save_to_disk);
    let t = Instant::now();
    let last = on_worker(move || {
        let png = match png {
            Some(p) => p,
            None => image::encode_png(&frame)?,
        };
        let path = if save {
            let (bytes, ext) = encode_for_disk(&frame, &png, &opts)?;
            Some(write_capture(&dir, &bytes, ext)?)
        } else {
            None
        };
        let copied = copy
            && match clipboard::copy_image(&png, &frame) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(error = %e, "copy to clipboard failed");
                    false
                }
            };
        Ok(LastCapture {
            png: Arc::new(png),
            width: frame.width,
            height: frame.height,
            frame: Arc::new(frame),
            path,
            copied,
        })
    })
    .await?;
    let info = last.info();
    tracing::info!(
        ms = t.elapsed().as_millis() as u64,
        ?intent,
        width = info.width,
        height = info.height,
        saved = info.path.is_some(),
        copied = info.copied,
        "screenshot delivered"
    );
    if let Some(path) = &last.path {
        let meta = crate::core::library::Meta {
            width: Some(last.width),
            height: Some(last.height),
            duration_ms: None,
        };
        crate::library::add(app, path, meta);
    }
    *state(app).screenshots.last.lock().unwrap() = Some(Arc::new(last));
    app.emit(COMPLETED_EVENT, &info)?;
    if settings.capture.show_preview {
        show_preview(app, PreviewKind::Image)?;
    } else {
        let s = crate::core::locale::tray_strings(crate::core::locale::current(settings.general.language));
        crate::notify::send(app, s.screenshot_saved, info.file_name.as_deref().unwrap_or_default());
    }
    Ok(())
}

/// A capture annotated in the selection overlay: PNG bytes rendered by the overlay.
pub async fn finish_annotated(app: AppHandle, png: Vec<u8>, intent: Intent) -> AppResult<()> {
    let had_session = state(&app).screenshots.session.lock().unwrap().take().is_some();
    close_overlays(&app);
    if !had_session {
        return Err(AppError::Runtime("no capture in progress".into()));
    }
    let (frame, png) = on_worker(move || {
        let rgba = ::image::load_from_memory_with_format(&png, ::image::ImageFormat::Png)
            .map_err(|e| AppError::Platform(format!("invalid PNG from overlay: {e}")))?
            .to_rgba8();
        let (w, h) = rgba.dimensions();
        let mut bgra = rgba.into_raw();
        for px in bgra.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        Ok((Frame::new(w, h, bgra)?, png))
    })
    .await?;
    deliver(&app, frame, Some(png), intent).await
}

/// Saves the last capture when automatic saving is off and the user asks from the preview.
pub fn save_last(app: &AppHandle) -> AppResult<CaptureInfo> {
    let st = state(app);
    let mut guard = st.screenshots.last.lock().unwrap();
    let last = guard.as_ref().ok_or_else(|| AppError::Runtime("no capture".into()))?;
    if last.path.is_some() {
        return Ok(last.info());
    }
    let dir = st.settings.get().storage.screenshots_dir;
    std::fs::create_dir_all(&dir)?;
    let opts = st.settings.get().capture;
    let (bytes, ext) = encode_for_disk(&last.frame, &last.png, &opts)?;
    let path = write_capture(&dir, &bytes, ext)?;
    let updated = Arc::new(LastCapture {
        png: Arc::clone(&last.png),
        frame: Arc::clone(&last.frame),
        width: last.width,
        height: last.height,
        path: Some(path),
        copied: last.copied,
    });
    let info = updated.info();
    if let Some(path) = &updated.path {
        let meta = crate::core::library::Meta {
            width: Some(updated.width),
            height: Some(updated.height),
            duration_ms: None,
        };
        crate::library::add(app, path, meta);
    }
    *guard = Some(updated);
    Ok(info)
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PreviewKind {
    Image,
    Video,
}

/// Small floating card in the bottom-right corner of the display under the pointer.
pub fn show_preview(app: &AppHandle, kind: PreviewKind) -> AppResult<()> {
    if let Some(w) = app.get_webview_window(PREVIEW) {
        app.emit_to(PREVIEW, "capture://preview-refresh", kind)?;
        let _ = w.show();
        return Ok(());
    }
    const W: f64 = 300.0;
    const H: f64 = 236.0;
    const MARGIN: f64 = 18.0;
    let st = state(app);
    let displays = st.backend.displays().unwrap_or_default();
    let at = st.backend.display_at_cursor();
    let d = displays
        .iter()
        .find(|d| Some(&d.id) == at.as_ref())
        .or_else(|| displays.iter().find(|d| d.is_primary));
    // Work areas are points on macOS and physical pixels on Windows; Tauri positions are logical.
    let (x, y) = match d {
        Some(d) => {
            let s = if cfg!(windows) { d.scale_factor } else { 1.0 };
            let a = d.work_area;
            ((a.x + a.width) / s - W - MARGIN, (a.y + a.height) / s - H - MARGIN)
        }
        None => (MARGIN, MARGIN),
    };
    let kind_param = match kind {
        PreviewKind::Image => "image",
        PreviewKind::Video => "video",
    };
    let url = WebviewUrl::App(format!("preview.html?kind={kind_param}").into());
    let window = WebviewWindowBuilder::new(app, PREVIEW, url)
        .title("FrameLy")
        .inner_size(W, H)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .shadow(false)
        .visible(false)
        .visible_on_all_workspaces(true)
        .build()?;
    chrome::configure_floating(&window);
    Ok(())
}

pub const FAILED_EVENT: &str = "capture://failed";

/// Fire-and-forget entry point for the tray and (Stage 6) global shortcuts. Failures surface
/// in the main window, since there's no other UI to report them in.
pub fn trigger(app: &AppHandle, request: Request) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start(app.clone(), request).await {
            tracing::warn!(error = %e, "screenshot failed");
            crate::windows::show_main(&app);
            let _ = app.emit(FAILED_EVENT, &e);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_follow_the_toolbar_and_settings() {
        // (save, copy)
        assert_eq!(plan(Intent::Save, false, false), (true, false));
        assert_eq!(plan(Intent::Save, true, false), (true, true));
        assert_eq!(plan(Intent::Copy, false, false), (false, true));
        assert_eq!(plan(Intent::Copy, false, true), (true, true));
        assert_eq!(plan(Intent::Auto, true, true), (true, true));
        assert_eq!(plan(Intent::Auto, true, false), (false, true));
        assert_eq!(plan(Intent::Auto, false, false), (true, false), "never lost");
    }
}
