//! Native window creation and management. Each window kind is built here so platform
//! chrome (macOS vibrancy/overlay title bar, Windows native frame) stays out of the UI code.

use crate::core::{errors::AppResult, settings::Theme};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const MAIN: &str = "main";

fn to_tauri_theme(t: Theme) -> Option<tauri::Theme> {
    match t {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    }
}

/// Main window size for a screen work area (logical points): most of the screen, so the library
/// and transcripts have room, but never smaller than the classic size when the screen allows it.
pub fn main_size(work_w: f64, work_h: f64) -> (f64, f64) {
    let fit = |share: f64, base: f64, max: f64, avail: f64| (avail * share).max(base.min(avail)).min(max).min(avail);
    (fit(0.86, 1120.0, 1760.0, work_w), fit(0.88, 760.0, 1100.0, work_h))
}

/// Builds the main window hidden; the UI calls `show()` after its first paint to avoid a blank flash.
pub fn create_main(app: &AppHandle, theme: Theme) -> AppResult<WebviewWindow> {
    let (w, h) = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let area = m.work_area().size.to_logical::<f64>(m.scale_factor());
            main_size(area.width, area.height)
        })
        .unwrap_or((1120.0, 760.0));
    let builder = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("main.html".into()))
        .title("FrameLy")
        .inner_size(w, h)
        .min_inner_size(640.0, 520.0);
    let window = app_window(builder, theme)?;
    // Debug aid: `FRAMELY_DEVTOOLS=1 pnpm tauri dev` shows the window immediately and opens
    // the Web Inspector, even if the UI fails before it can show itself.
    #[cfg(debug_assertions)]
    if std::env::var_os("FRAMELY_DEVTOOLS").is_some() {
        let _ = window.show();
        window.open_devtools();
    }
    Ok(window)
}

/// Shared chrome for FrameLy's full windows (main, editor): hidden until first paint and, on
/// macOS, the native traffic lights over FrameLy's own painted background (the brand's dark
/// atmosphere replaces the system vibrancy material).
pub fn app_window(
    builder: WebviewWindowBuilder<'_, tauri::Wry, AppHandle>,
    theme: Theme,
) -> AppResult<WebviewWindow> {
    let builder = builder.center().visible(false).theme(to_tauri_theme(theme));

    #[cfg(target_os = "macos")]
    let builder = {
        use tauri::{LogicalPosition, TitleBarStyle};
        builder
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(LogicalPosition::new(18.0, 24.0))
            .background_color(tauri::window::Color(8, 13, 24, 255))
    };

    Ok(builder.build()?)
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Hides the main window — safely from full screen too.
pub fn hide_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        crate::platform::chrome::hide_window(&w);
    }
}

pub fn apply_theme(app: &AppHandle, theme: Theme) {
    for w in app.webview_windows().values() {
        if let Err(e) = w.set_theme(to_tauri_theme(theme)) {
            tracing::warn!(window = w.label(), error = %e, "failed to apply theme");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::main_size;

    #[test]
    fn main_window_follows_the_screen() {
        // MacBook Air 13" (1470×919 work area): most of the screen.
        let (w, h) = main_size(1470.0, 919.0);
        assert!((1260.0..=1270.0).contains(&w) && (800.0..=815.0).contains(&h), "{w}×{h}");
        // Large monitor: capped so it doesn't become a wall of empty space.
        assert_eq!(main_size(3000.0, 1600.0), (1760.0, 1100.0));
        // Small screen: never larger than the available area.
        assert_eq!(main_size(1024.0, 700.0), (1024.0, 700.0));
        // 1920×1040: generous but not full screen.
        let (w, h) = main_size(1920.0, 1040.0);
        assert!(w < 1920.0 && h < 1040.0 && w > 1600.0 && h > 900.0, "{w}×{h}");
    }
}
