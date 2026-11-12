//! User feedback outside the windows: system notifications (used when no preview card is shown)
//! and short sounds for captures and recordings — each behind its own setting.

use crate::AppState;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

pub fn send(app: &AppHandle, title: &str, body: &str) {
    if !app.state::<AppState>().settings.get().general.notifications {
        return;
    }
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        tracing::debug!(error = %e, "notification not shown");
    }
}

/// Plays a feedback cue unless the user turned sounds off. Returns whether it played.
pub fn cue(app: &AppHandle, cue: crate::core::sounds::Cue) -> bool {
    let on = app.state::<AppState>().settings.get().general.sounds;
    if on {
        crate::platform::sound::play_wav(crate::core::sounds::wav_bytes(cue));
    }
    on
}
