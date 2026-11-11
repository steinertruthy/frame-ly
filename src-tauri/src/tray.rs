//! System tray / menu bar icon. Stage 0: open/quit. The rich quick panel arrives in Stage 6.

use crate::{
    core::{errors::AppResult, locale, settings::Language},
    windows,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Runtime,
};

const TRAY_ID: &str = "framely";

fn build_menu<R: Runtime>(app: &AppHandle<R>, language: Language) -> AppResult<Menu<R>> {
    let s = locale::tray_strings(locale::current(language));
    let open = MenuItem::with_id(app, "open", s.open, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", s.quit, true, Some("CmdOrCtrl+Q"))?;
    Ok(Menu::with_items(
        app,
        &[&open, &PredefinedMenuItem::separator(app)?, &quit],
    )?)
}

pub fn create(app: &AppHandle, language: Language) -> AppResult<()> {
    // macOS: monochrome template glyph adapts to light/dark menu bars.
    // Windows: the full-color app icon reads well on both taskbar themes.
    #[cfg(target_os = "macos")]
    let icon = Image::from_bytes(include_bytes!("../icons/tray-template.png"))?;
    #[cfg(not(target_os = "macos"))]
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip(locale::tray_strings(locale::current(language)).tooltip)
        .menu(&build_menu(app, language)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => windows::show_main(app),
            "quit" => {
                tracing::info!("quit requested from tray");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                windows::toggle_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

pub fn refresh_labels(app: &AppHandle, language: Language) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match build_menu(app, language) {
        Ok(menu) => {
            let _ = tray.set_menu(Some(menu));
        }
        Err(e) => tracing::warn!(error = %e, "failed to rebuild tray menu"),
    }
}
