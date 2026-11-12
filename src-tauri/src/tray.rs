//! System tray / menu bar icon. Idle: capture and record actions. While recording: a red
//! (or amber when paused) icon with the elapsed time and stop/pause controls; a left click
//! stops the recording. The rich quick panel arrives in Stage 6.

use crate::{
    core::{errors::AppResult, locale, recorder::Phase, settings::Language},
    panel, recording, screenshot,
    screenshot::{Request, SelectMode},
    windows, AppState,
};
use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

const TRAY_ID: &str = "framely";

/// Last phase rendered, so the menu and icon are rebuilt only on real changes.
static SHOWN: Mutex<Option<Phase>> = Mutex::new(None);

fn language(app: &AppHandle) -> Language {
    app.state::<AppState>().settings.get().general.language
}

fn build_menu(app: &AppHandle, language: Language, phase: Phase) -> AppResult<Menu<tauri::Wry>> {
    let s = locale::tray_strings(locale::current(language));
    let item = |id: &str, text: &str| MenuItem::with_id(app, id, text, true, None::<&str>);
    let sep = || PredefinedMenuItem::separator(app);
    let open = item("open", s.open)?;
    let quit = MenuItem::with_id(app, "quit", s.quit, true, Some("CmdOrCtrl+Q"))?;
    let menu = match phase {
        Phase::Recording | Phase::Paused | Phase::Countdown | Phase::Finalizing => {
            let stop = item("rec-stop", s.stop)?;
            let toggle = item("rec-toggle", if phase == Phase::Paused { s.resume } else { s.pause })?;
            toggle.set_enabled(matches!(phase, Phase::Recording | Phase::Paused))?;
            Menu::with_items(app, &[&stop, &toggle, &sep()?, &open, &sep()?, &quit])?
        }
        Phase::Idle => {
            // Same hierarchy as the app: screenshot first, then recording, then the app.
            let supports = app.state::<AppState>().backend.supports_recording();
            let shot = MenuItem::with_id(app, "capture-region", s.screenshot, true, None::<&str>)?;
            let screen = item("capture-screen", s.capture_screen)?;
            let window = item("capture-window", s.capture_window)?;
            let rec_screen = item("rec-screen", s.record_screen)?;
            let rec_window = item("rec-window", s.record_window)?;
            let rec_region = item("rec-region", s.record_region)?;
            let settings = item("settings", s.settings)?;
            if supports {
                Menu::with_items(
                    app,
                    &[
                        &shot,
                        &screen,
                        &window,
                        &sep()?,
                        &rec_screen,
                        &rec_window,
                        &rec_region,
                        &sep()?,
                        &open,
                        &settings,
                        &sep()?,
                        &quit,
                    ],
                )?
            } else {
                Menu::with_items(app, &[&shot, &screen, &window, &sep()?, &open, &settings, &sep()?, &quit])?
            }
        }
    };
    Ok(menu)
}

fn idle_icon() -> AppResult<Image<'static>> {
    // macOS: monochrome template glyph adapts to light/dark menu bars.
    // Windows: the full-color app icon reads well on both taskbar themes.
    #[cfg(target_os = "macos")]
    return Ok(Image::from_bytes(include_bytes!("../icons/tray-template.png"))?);
    #[cfg(not(target_os = "macos"))]
    return Ok(Image::from_bytes(include_bytes!("../icons/32x32.png"))?);
}

fn on_menu(app: &AppHandle, id: &str) {
    let report = |r: crate::core::errors::AppResult<()>| {
        if let Err(e) = r {
            tracing::warn!(error = %e, "tray action failed");
        }
    };
    match id {
        "open" => windows::show_main(app),
        "settings" => {
            windows::show_main(app);
            let _ = tauri::Emitter::emit_to(app, windows::MAIN, "main://navigate", "settings");
        }
        "capture-region" => screenshot::trigger(app, Request::Select { mode: SelectMode::Region }),
        "capture-window" => screenshot::trigger(app, Request::Select { mode: SelectMode::Window }),
        "capture-screen" => screenshot::trigger(app, Request::Display { display_id: None }),
        "rec-region" => recording::trigger(app, recording::Request::Select { mode: SelectMode::Region }),
        "rec-window" => recording::trigger(app, recording::Request::Select { mode: SelectMode::Window }),
        "rec-screen" => recording::trigger(app, recording::Request::Display { display_id: None }),
        "rec-toggle" => report(recording::toggle_pause(app)),
        "rec-stop" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move { report(recording::stop(app).await) });
        }
        "quit" => {
            tracing::info!("quit requested from tray");
            app.exit(0);
        }
        _ => {}
    }
}

pub fn create(app: &AppHandle, language: Language) -> AppResult<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(idle_icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip(locale::tray_strings(locale::current(language)).tooltip)
        .menu(&build_menu(app, language, Phase::Idle)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            // Left click: the quick panel (idle actions, or live controls while recording).
            // Right click: the native menu, as a keyboard/accessibility-friendly fallback.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let pos = rect.position.to_physical::<f64>(1.0);
                let size = rect.size.to_physical::<f64>(1.0);
                panel::toggle(tray.app_handle(), Some((pos.x, pos.y, size.width, size.height)));
            }
        })
        .build(app)?;
    *SHOWN.lock().unwrap() = Some(Phase::Idle);
    Ok(())
}

/// Reflects the recording phase in the tray: icon, elapsed-time title (macOS) and menu.
pub fn show_recording(app: &AppHandle, phase: Phase, elapsed: &str) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let active = matches!(phase, Phase::Recording | Phase::Paused);
    let _ = tray.set_title(active.then_some(elapsed));
    let mut shown = SHOWN.lock().unwrap();
    if *shown == Some(phase) {
        return;
    }
    *shown = Some(phase);
    let icon: AppResult<Image<'static>> = match phase {
        Phase::Recording | Phase::Countdown | Phase::Finalizing => {
            Image::from_bytes(include_bytes!("../icons/tray-recording.png")).map_err(Into::into)
        }
        Phase::Paused => Image::from_bytes(include_bytes!("../icons/tray-paused.png")).map_err(Into::into),
        Phase::Idle => idle_icon(),
    };
    if let Ok(icon) = icon {
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_icon_as_template(phase == Phase::Idle && cfg!(target_os = "macos"));
    }
    match build_menu(app, language(app), phase) {
        Ok(menu) => {
            let _ = tray.set_menu(Some(menu));
        }
        Err(e) => tracing::warn!(error = %e, "failed to rebuild tray menu"),
    }
}

pub fn refresh_labels(app: &AppHandle, language: Language) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else { return };
    let phase = recording::status(app).phase;
    match build_menu(app, language, phase) {
        Ok(menu) => {
            let _ = tray.set_menu(Some(menu));
        }
        Err(e) => tracing::warn!(error = %e, "failed to rebuild tray menu"),
    }
}
