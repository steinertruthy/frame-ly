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

/// Builds the main window hidden; the UI calls `show()` after its first paint to avoid a blank flash.
pub fn create_main(app: &AppHandle, theme: Theme) -> AppResult<WebviewWindow> {
    let builder = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("main.html".into()))
        .title("FrameLy")
        .inner_size(1120.0, 760.0)
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

pub fn apply_theme(app: &AppHandle, theme: Theme) {
    for w in app.webview_windows().values() {
        if let Err(e) = w.set_theme(to_tauri_theme(theme)) {
            tracing::warn!(window = w.label(), error = %e, "failed to apply theme");
        }
    }
}
