//! System notifications, used when no preview card is shown (respecting the user's setting).

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
