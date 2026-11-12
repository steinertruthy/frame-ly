//! The menu-bar / system-tray quick panel: a compact window anchored to the tray icon
//! (below it on macOS, above it on a bottom Windows taskbar), hidden when it loses focus.

use crate::{core::errors::AppResult, platform::chrome};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub const PANEL: &str = "panel";
pub const SHOWN_EVENT: &str = "panel://shown";
const WIDTH: f64 = 340.0;
const HEIGHT: f64 = 440.0;
const GAP: f64 = 6.0;

/// Tray icon rectangle (physical px) from the last click, to anchor shortcut-opened panels too.
static ANCHOR: Mutex<Option<(f64, f64, f64, f64)>> = Mutex::new(None);
/// When the panel last hid itself on blur — a click on the tray icon blurs the panel just
/// before the click event arrives, which must not reopen it.
static LAST_BLUR_HIDE: Mutex<Option<Instant>> = Mutex::new(None);

/// Builds the panel hidden at startup so opening it is instant.
pub fn create(app: &AppHandle) -> AppResult<()> {
    let builder = WebviewWindowBuilder::new(app, PANEL, WebviewUrl::App("panel.html".into()))
        .title("FrameLy")
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .visible_on_all_workspaces(true);
    #[cfg(target_os = "macos")]
    let builder = {
        use tauri::{utils::config::WindowEffectsConfig, window::Effect};
        builder.shadow(true).effects(WindowEffectsConfig {
            effects: vec![Effect::Popover],
            radius: Some(14.0),
            ..Default::default()
        })
    };
    #[cfg(windows)]
    let builder = builder.shadow(false);
    let window = builder.build()?;
    chrome::configure_panel(&window);
    Ok(())
}

/// `anchor` = tray icon rect (physical x, y, w, h) when opened by clicking it.
pub fn toggle(app: &AppHandle, anchor: Option<(f64, f64, f64, f64)>) {
    let Some(w) = app.get_webview_window(PANEL) else { return };
    if w.is_visible().unwrap_or(false) {
        hide(app);
        return;
    }
    let just_hidden =
        LAST_BLUR_HIDE.lock().unwrap().is_some_and(|t| t.elapsed() < Duration::from_millis(300));
    if just_hidden && anchor.is_some() {
        return;
    }
    show(app, anchor);
}

pub fn show(app: &AppHandle, anchor: Option<(f64, f64, f64, f64)>) {
    let Some(w) = app.get_webview_window(PANEL) else { return };
    if let Some(a) = anchor {
        *ANCHOR.lock().unwrap() = Some(a);
    }
    if let Some(pos) = position(app) {
        let _ = w.set_position(pos);
    }
    let _ = app.emit_to(PANEL, SHOWN_EVENT, ());
    chrome::show_panel(&w);
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PANEL) {
        let _ = w.hide();
    }
}

/// Called when the panel loses focus.
pub fn on_blur(app: &AppHandle) {
    let visible = app.get_webview_window(PANEL).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    if visible {
        *LAST_BLUR_HIDE.lock().unwrap() = Some(Instant::now());
        hide(app);
    }
}

/// Centered under (or above) the tray icon and kept inside the display's work area. Without a
/// known icon position (opened by shortcut before any click), top-right of the primary display.
///
/// Uses FrameLy's own display list (work area from `NSScreen.visibleFrame` / `rcWork`), whose
/// units are known: points on macOS, physical pixels on Windows.
fn position(app: &AppHandle) -> Option<PhysicalPosition<i32>> {
    let displays = app.state::<crate::AppState>().backend.displays().ok()?;
    let anchor = *ANCHOR.lock().unwrap();
    // OS units → physical pixels for each display.
    let to_px = |d: &crate::platform::Display| if cfg!(target_os = "macos") { d.scale_factor } else { 1.0 };
    let display = anchor
        .and_then(|(x, y, _, _)| {
            displays.iter().find(|d| {
                let (k, b) = (to_px(d), d.bounds);
                x >= b.x * k && x < (b.x + b.width) * k && y >= b.y * k && y < (b.y + b.height) * k
            })
        })
        .or_else(|| displays.iter().find(|d| d.is_primary))?;
    let k = to_px(display);
    let scale = display.scale_factor;
    let (pw, ph, gap) = (WIDTH * scale, HEIGHT * scale, GAP * scale);
    let a = display.work_area;
    let (ax, ay, aw, ah) = (a.x * k, a.y * k, a.width * k, a.height * k);
    let (x, y) = match anchor {
        Some((x, y, w, h)) => {
            let below = y + h / 2.0 < ay + ah / 2.0; // icon in the top half → menu bar
            (x + w / 2.0 - pw / 2.0, if below { y + h + gap } else { y - ph - gap })
        }
        None => (ax + aw - pw - gap * 2.0, ay + gap),
    };
    let x = x.clamp(ax + gap, (ax + aw - pw - gap).max(ax + gap));
    let y = y.clamp(ay, (ay + ah - ph).max(ay));
    Some(PhysicalPosition::new(x.round() as i32, y.round() as i32))
}
