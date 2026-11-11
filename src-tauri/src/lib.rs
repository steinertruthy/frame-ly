//! FrameLy application entry: wires plugins, state, windows, tray and IPC commands.

mod commands;
mod core;
mod platform;
mod tray;
mod windows;

use crate::core::settings::{DefaultDirs, SettingsStore};
use std::{path::PathBuf, sync::Arc};
use tauri::{Manager, RunEvent, WindowEvent};

/// Process-wide state shared with IPC commands.
pub struct AppState {
    pub settings: SettingsStore,
    pub backend: Arc<dyn platform::CaptureBackend>,
    pub log_dir: PathBuf,
    _log_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

pub fn run() {
    let app = tauri::Builder::default()
        // Must be first: a second launch focuses the running instance instead of starting another.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            windows::show_main(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
                log_dir,
                _log_guard: log_guard,
            });

            windows::create_main(&handle, initial.appearance.theme)?;
            tray::create(&handle, initial.general.language)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the main window keeps FrameLy alive in the tray (recordings must survive it).
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == windows::MAIN {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
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
