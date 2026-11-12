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

struct Session {
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
            deliver(&app, frame).await
        }
        Request::Select { mode } => {
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
                Some(Session { mode, displays: displays.clone(), frames, windows });
            for (i, d) in displays.iter().enumerate() {
                if let Err(e) = open_overlay(&app, i, d) {
                    cancel(&app);
                    return Err(e);
                }
            }
            Ok(())
        }
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
    Ok(OverlayInit { mode: s.mode, frame_width: frame.width, frame_height: frame.height, display, windows })
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
    let frame = {
        let st = state(&app);
        let mut guard = st.screenshots.session.lock().unwrap();
        let s = guard.take().ok_or_else(|| AppError::Runtime("no capture in progress".into()))?;
        s.frames
            .get(&display_id)
            .cloned()
            .ok_or_else(|| AppError::Runtime(format!("unknown display {display_id}")))?
    };
    close_overlays(&app);
    let rect = image::css_to_pixels(selection, viewport, (frame.width, frame.height));
    let cropped = on_worker(move || frame.crop(rect)).await?;
    deliver(&app, cropped).await
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
    deliver(&app, frame).await
}

fn crop_window_from_session(s: &Session, window_id: &str) -> Option<Frame> {
    let w = s.windows.iter().find(|w| w.id == window_id)?;
    let (cx, cy) = (w.bounds.x + w.bounds.width / 2.0, w.bounds.y + w.bounds.height / 2.0);
    let d = s.displays.iter().find(|d| {
        let b = d.bounds;
        cx >= b.x && cx < b.x + b.width && cy >= b.y && cy < b.y + b.height
    })?;
    let frame = s.frames.get(&d.id)?;
    let sel = (w.bounds.x - d.bounds.x, w.bounds.y - d.bounds.y, w.bounds.width, w.bounds.height);
    frame.crop(image::css_to_pixels(sel, (d.bounds.width, d.bounds.height), (frame.width, frame.height))).ok()
}

/// Encodes, saves and/or copies the capture according to settings, then shows the preview.
async fn deliver(app: &AppHandle, frame: Frame) -> AppResult<()> {
    let settings = state(app).settings.get();
    let dir = settings.storage.screenshots_dir.clone();
    let opts = settings.capture.clone();
    let t = Instant::now();
    let last = on_worker(move || {
        let t = Instant::now();
        let png = image::encode_png(&frame)?;
        let encode_ms = t.elapsed().as_millis() as u64;
        let t = Instant::now();
        let path = if opts.save_to_disk {
            std::fs::create_dir_all(&dir)?;
            let path = paths::unique_path(&dir, &paths::capture_file_name(&chrono::Local::now(), "png"));
            std::fs::write(&path, &png)?;
            Some(path)
        } else {
            None
        };
        let save_ms = t.elapsed().as_millis() as u64;
        let t = Instant::now();
        let copied = opts.copy_to_clipboard
            && match clipboard::copy_image(&png, &frame) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(error = %e, "copy to clipboard failed");
                    false
                }
            };
        tracing::info!(
            encode_ms,
            save_ms,
            copy_ms = t.elapsed().as_millis() as u64,
            bytes = png.len(),
            "delivery steps"
        );
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
        width = info.width,
        height = info.height,
        saved = info.path.is_some(),
        copied = info.copied,
        "screenshot delivered"
    );
    *state(app).screenshots.last.lock().unwrap() = Some(Arc::new(last));
    app.emit(COMPLETED_EVENT, &info)?;
    if settings.capture.show_preview {
        show_preview(app)?;
    }
    Ok(())
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
    let path = paths::unique_path(&dir, &paths::capture_file_name(&chrono::Local::now(), "png"));
    std::fs::write(&path, last.png.as_slice())?;
    let updated = Arc::new(LastCapture {
        png: Arc::clone(&last.png),
        frame: Arc::clone(&last.frame),
        width: last.width,
        height: last.height,
        path: Some(path),
        copied: last.copied,
    });
    let info = updated.info();
    *guard = Some(updated);
    Ok(info)
}

/// Small floating card in the bottom-right corner of the display under the pointer.
fn show_preview(app: &AppHandle) -> AppResult<()> {
    if let Some(w) = app.get_webview_window(PREVIEW) {
        app.emit_to(PREVIEW, "capture://preview-refresh", ())?;
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
    let window = WebviewWindowBuilder::new(app, PREVIEW, WebviewUrl::App("preview.html".into()))
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
