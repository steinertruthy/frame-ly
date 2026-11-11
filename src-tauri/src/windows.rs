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
        .inner_size(1120.0, 740.0)
        .min_inner_size(880.0, 580.0)
        .center()
        .visible(false)
        .theme(to_tauri_theme(theme));

    #[cfg(target_os = "macos")]
    let builder = {
        use tauri::{
            utils::config::WindowEffectsConfig,
            window::{Effect, EffectState},
            LogicalPosition, TitleBarStyle,
        };
        builder
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(LogicalPosition::new(18.0, 24.0))
            .transparent(true)
            .effects(WindowEffectsConfig {
                effects: vec![Effect::Sidebar],
                state: Some(EffectState::FollowsWindowActiveState),
                radius: None,
                color: None,
                ..Default::default()
            })
    };

    let window = builder.build()?;
    // Debug aid: `FRAMELY_DEVTOOLS=1 pnpm tauri dev` shows the window immediately and opens
    // the Web Inspector, even if the UI fails before it can show itself.
    #[cfg(debug_assertions)]
    if std::env::var_os("FRAMELY_DEVTOOLS").is_some() {
        let _ = window.show();
        window.open_devtools();
    }
    Ok(window)
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let visible = w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false);
        if visible && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_main(app);
        }
    }
}

pub fn apply_theme(app: &AppHandle, theme: Theme) {
    for w in app.webview_windows().values() {
        if let Err(e) = w.set_theme(to_tauri_theme(theme)) {
            tracing::warn!(window = w.label(), error = %e, "failed to apply theme");
        }
    }
}
