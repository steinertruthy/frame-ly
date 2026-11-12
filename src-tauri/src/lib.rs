//! FrameLy application entry: wires plugins, state, windows, tray and IPC commands.

mod commands;
mod core;
mod panel;
mod platform;
mod protocol;
mod recording;
mod screenshot;
mod shortcuts;
mod tray;
mod windows;

use crate::core::settings::{DefaultDirs, SettingsStore};
use std::{path::PathBuf, sync::Arc};
use tauri::{Manager, RunEvent, WindowEvent};

/// Process-wide state shared with IPC commands.
pub struct AppState {
    pub settings: SettingsStore,
    pub backend: Arc<dyn platform::CaptureBackend>,
    pub screenshots: screenshot::Screenshots,
    pub recordings: recording::Recordings,
    pub log_dir: PathBuf,
    _log_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

enum CliAction {
    Capture(screenshot::Request),
    Cancel,
    Record(recording::Request),
    StopRecording,
    TogglePause,
    Panel,
}

/// `framely --panel`, `framely --capture region|window|screen|cancel` and
/// `framely --record region|window|screen|stop|pause` (forwarded to the running instance) —
/// lets people bind FrameLy in launchers and automation tools.
fn cli_action(argv: &[String]) -> Option<CliAction> {
    use screenshot::{Request, SelectMode};
    if argv.iter().any(|a| a == "--panel") {
        return Some(CliAction::Panel);
    }
    if let Some(i) = argv.iter().position(|a| a == "--record") {
        return match argv.get(i + 1).map(String::as_str) {
            Some("region") => {
                Some(CliAction::Record(recording::Request::Select { mode: SelectMode::Region }))
            }
            Some("window") => {
                Some(CliAction::Record(recording::Request::Select { mode: SelectMode::Window }))
            }
            Some("screen") => Some(CliAction::Record(recording::Request::Display { display_id: None })),
            Some("stop") => Some(CliAction::StopRecording),
            Some("pause") => Some(CliAction::TogglePause),
            _ => None,
        };
    }
    let i = argv.iter().position(|a| a == "--capture")?;
    match argv.get(i + 1).map(String::as_str) {
        Some("region") => Some(CliAction::Capture(Request::Select { mode: SelectMode::Region })),
        Some("window") => Some(CliAction::Capture(Request::Select { mode: SelectMode::Window })),
        Some("screen") => Some(CliAction::Capture(Request::Display { display_id: None })),
        Some("cancel") => Some(CliAction::Cancel),
        _ => None,
    }
}

fn run_cli_action(app: &tauri::AppHandle, action: CliAction) {
    match action {
        CliAction::Capture(request) => screenshot::trigger(app, request),
        CliAction::Cancel => screenshot::cancel(app),
        CliAction::Record(request) => recording::trigger(app, request),
        CliAction::StopRecording => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = recording::stop(app).await {
                    tracing::warn!(error = %e, "stop from CLI failed");
                }
            });
        }
        CliAction::TogglePause => {
            if let Err(e) = recording::toggle_pause(app) {
                tracing::warn!(error = %e, "pause from CLI failed");
            }
        }
        CliAction::Panel => panel::toggle(app, None),
    }
}

pub fn run() {
    let app = tauri::Builder::default()
        // Must be first: a second launch focuses the running instance instead of starting another.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| match cli_action(&argv) {
            Some(action) => run_cli_action(app, action),
            None => windows::show_main(app),
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcuts::plugin())
        // In-memory images for overlays and the preview: framely://localhost/frame/<display>
        // (frozen frame, BMP) and framely://localhost/last (last capture, PNG).
        .register_asynchronous_uri_scheme_protocol("framely", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let path = request.uri().path().to_string();
            std::thread::spawn(move || responder.respond(protocol::respond(&app, &path)));
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let log_dir = app.path().app_log_dir()?;
            let log_guard = core::logging::init(&log_dir);
            tracing::info!(
                version = %app.package_info().version,
                os = std::env::consts::OS,
                arch = std::env::consts::ARCH,
                "FrameLy starting"
            );

            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let settings = SettingsStore::load(settings_path, &DefaultDirs::from_os())?;
            let initial = settings.get();
            app.manage(AppState {
                settings,
                backend: Arc::from(platform::backend()),
                screenshots: Default::default(),
                recordings: Default::default(),
                log_dir,
                _log_guard: log_guard,
            });

            recording::recover(&handle);
            windows::create_main(&handle, initial.appearance.theme)?;
            if let Some(action) = cli_action(&std::env::args().collect::<Vec<_>>()) {
                run_cli_action(&handle, action);
            }
            tray::create(&handle, initial.general.language)?;
            panel::create(&handle)?;
            let supports_recording = handle.state::<AppState>().backend.supports_recording();
            shortcuts::apply(&handle, &initial.shortcuts, supports_recording);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Closing the main window keeps FrameLy alive in the tray (recordings must survive it).
            WindowEvent::CloseRequested { api, .. } if window.label() == windows::MAIN => {
                api.prevent_close();
                let _ = window.hide();
            }
            // The quick panel behaves like a popover: clicking anywhere else dismisses it.
            WindowEvent::Focused(false) if window.label() == panel::PANEL => {
                panel::on_blur(window.app_handle())
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_shortcuts,
            commands::system::app_info,
            commands::system::open_folder,
            commands::system::log_ui,
            commands::capture::list_displays,
            commands::capture::list_windows,
            commands::capture::list_audio_inputs,
            commands::capture::get_permissions,
            commands::capture::request_permission,
            commands::capture::open_permission_settings,
            commands::capture::relaunch,
            commands::screenshot::take_screenshot,
            commands::screenshot::overlay_init,
            commands::screenshot::finish_region,
            commands::screenshot::finish_window,
            commands::screenshot::cancel_capture,
            commands::screenshot::last_capture,
            commands::screenshot::copy_last_capture,
            commands::screenshot::save_last_capture,
            commands::screenshot::reveal_last_capture,
            commands::screenshot::open_last_capture,
            commands::screenshot::close_preview,
            commands::recording::start_recording,
            commands::recording::pause_recording,
            commands::recording::resume_recording,
            commands::recording::stop_recording,
            commands::recording::cancel_countdown,
            commands::recording::recording_status,
            commands::recording::recovered_recordings,
            commands::recording::last_recording,
            commands::recording::reveal_last_recording,
            commands::recording::open_last_recording,
            commands::recording::copy_last_recording,
            commands::recording::mic_test_start,
            commands::recording::mic_test_level,
            commands::recording::mic_test_stop,
            commands::system::panel_hide,
            commands::system::show_main_window,
            commands::system::shortcut_status,
            commands::system::suspend_shortcuts,
            commands::system::resume_shortcuts,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build FrameLy");

    app.run(|handle, event| match event {
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => windows::show_main(handle),
        RunEvent::Exit => tracing::info!("FrameLy exited"),
        _ => {
            let _ = handle;
        }
    });
}
