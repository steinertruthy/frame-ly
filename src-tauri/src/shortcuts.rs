//! Global shortcut registration. The OS can refuse a chord another app already holds; that is
//! reported per action (`shortcuts://status`) so Settings can show it instead of failing silently.

use crate::{
    core::shortcuts::{Action, Shortcuts},
    panel, recording,
    screenshot::{self, SelectMode},
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Mutex,
};
use tauri::{plugin::TauriPlugin, AppHandle, Emitter, Wry};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub const STATUS_EVENT: &str = "shortcuts://status";

/// Registration error per action (`None` = registered fine). Absent = disabled.
pub type Status = BTreeMap<Action, Option<String>>;

static REGISTERED: Mutex<Option<HashMap<u32, Action>>> = Mutex::new(None);
static STATUS: Mutex<Option<Status>> = Mutex::new(None);

pub fn plugin() -> TauriPlugin<Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let action = REGISTERED.lock().unwrap().as_ref().and_then(|m| m.get(&shortcut.id()).copied());
            if let Some(action) = action {
                tracing::info!(?action, "global shortcut");
                run(app, action);
            }
        })
        .build()
}

pub fn run(app: &AppHandle, action: Action) {
    use recording::Request as Rec;
    use screenshot::Request as Shot;
    match action {
        Action::OpenPanel => panel::toggle(app, None),
        Action::CaptureRegion => screenshot::trigger(app, Shot::Select { mode: SelectMode::Region }),
        Action::CaptureWindow => screenshot::trigger(app, Shot::Select { mode: SelectMode::Window }),
        Action::CaptureScreen => screenshot::trigger(app, Shot::Display { display_id: None }),
        Action::RecordRegion => recording::trigger(app, Rec::Select { mode: SelectMode::Region }),
        Action::RecordWindow => recording::trigger(app, Rec::Select { mode: SelectMode::Window }),
        Action::RecordScreen => recording::trigger(app, Rec::Display { display_id: None }),
        Action::StopRecording => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = recording::stop(app).await {
                    tracing::debug!(error = %e, "stop shortcut ignored");
                }
            });
        }
        Action::TogglePause => {
            if let Err(e) = recording::toggle_pause(app) {
                tracing::debug!(error = %e, "pause shortcut ignored");
            }
        }
    }
}

/// Replaces every registration with `set`. Recording actions are skipped where recording
/// isn't supported (so they never claim keys for nothing).
pub fn apply(app: &AppHandle, set: &Shortcuts, supports_recording: bool) {
    let gs = app.global_shortcut();
    if let Err(e) = gs.unregister_all() {
        tracing::warn!(error = %e, "could not clear global shortcuts");
    }
    let mut registered = HashMap::new();
    let mut status = Status::new();
    for (action, accel) in set.iter().filter(|(_, a)| !a.is_empty()) {
        let recording_action = matches!(
            action,
            Action::RecordRegion
                | Action::RecordWindow
                | Action::RecordScreen
                | Action::StopRecording
                | Action::TogglePause
        );
        if recording_action && !supports_recording {
            continue;
        }
        let result = accel
            .parse::<Shortcut>()
            .map_err(|e| e.to_string())
            .and_then(|s| gs.register(s).map(|_| s.id()).map_err(|e| e.to_string()));
        match result {
            Ok(id) => {
                registered.insert(id, *action);
                status.insert(*action, None);
            }
            Err(e) => {
                tracing::warn!(?action, %accel, error = %e, "global shortcut unavailable");
                status.insert(*action, Some(e));
            }
        }
    }
    tracing::info!(count = registered.len(), "global shortcuts registered");
    *REGISTERED.lock().unwrap() = Some(registered);
    *STATUS.lock().unwrap() = Some(status.clone());
    let _ = app.emit(STATUS_EVENT, &status);
}

/// While the user records a new shortcut in Settings, existing ones must not fire.
pub fn suspend(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
    *REGISTERED.lock().unwrap() = Some(HashMap::new());
}

pub fn status() -> Status {
    STATUS.lock().unwrap().clone().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::shortcuts::defaults;

    #[test]
    fn default_accelerators_parse_for_the_os_layer() {
        for (action, accel) in defaults().iter().filter(|(_, a)| !a.is_empty()) {
            assert!(accel.parse::<Shortcut>().is_ok(), "{action:?}: {accel}");
        }
        // Canonical forms produced by core::shortcuts::normalize also parse.
        for accel in ["Ctrl+Alt+F5", "Super+Alt+Space", "CmdOrCtrl+Shift+BracketLeft", "F13"] {
            assert!(accel.parse::<Shortcut>().is_ok(), "{accel}");
        }
    }
}
