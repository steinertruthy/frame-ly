//! Automatic updates (tauri-plugin-updater). FrameLy reads `latest.json` from the latest GitHub
//! release, verifies the package against the public key in tauri.conf.json, and installs it only
//! when no recording is in progress.
//!
//! - Background: a check shortly after launch and every few hours, unless turned off in Settings
//!   or blocked by "Local only" (which promises no connections).
//! - The UI only shows a quiet notice; nothing is downloaded until the person chooses to update.
//! - `FRAMELY_UPDATE_URL` (run time) points at another `latest.json`, for testing.

use crate::{
    core::errors::{AppError, AppResult},
    AppState,
};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// First background check after launch (lets startup and permission prompts settle).
const FIRST_CHECK: Duration = Duration::from_secs(45);
/// Background checks after that.
const INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
/// How often a pending install looks at the recorder again.
const RECORDING_POLL: Duration = Duration::from_secs(2);
pub const EVENT: &str = "updates://status";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Status {
    Idle,
    Checking,
    #[serde(rename_all = "camelCase")]
    UpToDate {
        checked_at: u64,
    },
    #[serde(rename_all = "camelCase")]
    Available {
        version: String,
        notes: Option<String>,
        date: Option<String>,
        dismissed: bool,
    },
    #[serde(rename_all = "camelCase")]
    Downloading {
        version: String,
        received: u64,
        total: Option<u64>,
    },
    /// Downloaded and verified; installs as soon as the recording ends.
    #[serde(rename_all = "camelCase")]
    WaitingForRecording {
        version: String,
    },
    #[serde(rename_all = "camelCase")]
    Installing {
        version: String,
    },
    /// "local_only" | "check_failed" | "download_failed" | "install_failed"
    #[serde(rename_all = "camelCase")]
    Error {
        reason: String,
        detail: Option<String>,
    },
}

#[derive(Default)]
pub struct Updates {
    status: Mutex<Option<Status>>,
    update: Mutex<Option<Update>>,
    bytes: Mutex<Option<Vec<u8>>>,
    busy: AtomicBool,
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn updates(app: &AppHandle) -> tauri::State<'_, Updates> {
    app.state::<Updates>()
}

pub fn status(app: &AppHandle) -> Status {
    updates(app).status.lock().unwrap().clone().unwrap_or(Status::Idle)
}

fn set(app: &AppHandle, s: Status) {
    *updates(app).status.lock().unwrap() = Some(s.clone());
    let _ = app.emit(EVENT, s);
}

fn local_only(app: &AppHandle) -> bool {
    app.state::<AppState>().settings.get().intelligence.local_only
}

fn recording_active(app: &AppHandle) -> bool {
    crate::recording::status(app).phase.is_active()
}

fn updater(app: &AppHandle) -> AppResult<tauri_plugin_updater::Updater> {
    let mut b = app.updater_builder();
    if let Some(url) = std::env::var("FRAMELY_UPDATE_URL").ok().filter(|u| !u.is_empty()) {
        let url = url.parse().map_err(|e| AppError::Runtime(format!("FRAMELY_UPDATE_URL: {e}")))?;
        b = b.endpoints(vec![url]).map_err(|e| AppError::Runtime(e.to_string()))?;
    }
    b.build().map_err(|e| AppError::Runtime(e.to_string()))
}

/// Background loop, started once from `setup`.
pub fn start(app: &AppHandle) {
    app.manage(Updates::default());
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        loop {
            let s = app.state::<AppState>().settings.get();
            if s.updates.auto_check && !s.intelligence.local_only {
                check(&app, false).await;
            }
            tokio::time::sleep(INTERVAL).await;
        }
    });
}

/// Looks for a newer version. `manual`: errors are shown (background checks stay quiet).
pub async fn check(app: &AppHandle, manual: bool) -> Status {
    let u = updates(app);
    if u.busy.load(Ordering::SeqCst) {
        return status(app);
    }
    if local_only(app) {
        let s = Status::Error { reason: "local_only".into(), detail: None };
        if manual {
            set(app, s.clone());
        }
        return s;
    }
    // Already found (and maybe dismissed): keep it, a manual check shows it again.
    if let Some(Status::Available { version, notes, date, .. }) = u.status.lock().unwrap().clone() {
        let s = Status::Available { version, notes, date, dismissed: !manual && is_dismissed(app) };
        set(app, s.clone());
        return s;
    }
    let previous = status(app);
    if manual {
        set(app, Status::Checking);
    }
    let result = match updater(app) {
        Ok(up) => up.check().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    let s = match result {
        Ok(Some(update)) => {
            tracing::info!(version = %update.version, "update available");
            let s = Status::Available {
                version: update.version.clone(),
                notes: update.body.clone().filter(|b| !b.trim().is_empty()),
                date: update.date.map(|d| d.to_string()),
                dismissed: false,
            };
            *u.update.lock().unwrap() = Some(update);
            *u.bytes.lock().unwrap() = None;
            s
        }
        Ok(None) => Status::UpToDate { checked_at: now() },
        Err(detail) => {
            tracing::warn!(%detail, "update check failed");
            if !manual {
                return previous;
            }
            Status::Error { reason: "check_failed".into(), detail: Some(detail) }
        }
    };
    set(app, s.clone());
    s
}

fn is_dismissed(app: &AppHandle) -> bool {
    matches!(status(app), Status::Available { dismissed: true, .. })
}

/// "Later": hides the notice until the next launch (Settings still shows it).
pub fn dismiss(app: &AppHandle) {
    if let Status::Available { version, notes, date, .. } = status(app) {
        set(app, Status::Available { version, notes, date, dismissed: true });
    }
}

/// Downloads (with progress), waits for any recording to end, installs and restarts.
pub async fn install(app: AppHandle) -> AppResult<()> {
    let u = updates(&app);
    if u.busy.swap(true, Ordering::SeqCst) {
        return Ok(()); // already on it
    }
    let result = install_inner(&app).await;
    updates(&app).busy.store(false, Ordering::SeqCst);
    if let Err(e) = &result {
        tracing::error!(error = %e, "update failed");
    }
    result
}

async fn install_inner(app: &AppHandle) -> AppResult<()> {
    let Some(update) = updates(app).update.lock().unwrap().clone() else {
        return Err(AppError::Runtime("no update to install".into()));
    };
    let version = update.version.clone();

    let cached = updates(app).bytes.lock().unwrap().clone();
    let bytes = match cached {
        Some(b) => b,
        None => {
            set(app, Status::Downloading { version: version.clone(), received: 0, total: None });
            let mut received = 0u64;
            let mut last = Instant::now();
            let progress_app = app.clone();
            let progress_version = version.clone();
            // Downloads and verifies the signature against the public key in tauri.conf.json.
            let bytes = update
                .download(
                    move |chunk, total| {
                        received += chunk as u64;
                        if last.elapsed() >= Duration::from_millis(120) || Some(received) == total {
                            last = Instant::now();
                            set(
                                &progress_app,
                                Status::Downloading { version: progress_version.clone(), received, total },
                            );
                        }
                    },
                    || {},
                )
                .await
                .map_err(|e| {
                    set(app, Status::Error { reason: "download_failed".into(), detail: Some(e.to_string()) });
                    AppError::Runtime(e.to_string())
                })?;
            *updates(app).bytes.lock().unwrap() = Some(bytes.clone());
            bytes
        }
    };

    // Never interrupt a recording: install once it has finished (and been saved).
    if recording_active(app) {
        set(app, Status::WaitingForRecording { version: version.clone() });
        while recording_active(app) {
            tokio::time::sleep(RECORDING_POLL).await;
        }
    }

    set(app, Status::Installing { version: version.clone() });
    tracing::info!(%version, "installing update");
    // On Windows this starts the installer (which updates in place and reopens FrameLy) and exits.
    update.install(bytes).map_err(|e| {
        set(app, Status::Error { reason: "install_failed".into(), detail: Some(e.to_string()) });
        AppError::Runtime(e.to_string())
    })?;
    app.restart();
}

/// `framely --update`: check and, if there is a newer version, install it.
pub fn check_and_install(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Status::Available { .. } = check(&app, true).await {
            let _ = install(app).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_for_the_ui() {
        let s = Status::Downloading { version: "0.3.0".into(), received: 10, total: Some(20) };
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            serde_json::json!({ "state": "downloading", "version": "0.3.0", "received": 10, "total": 20 })
        );
        let s = Status::WaitingForRecording { version: "0.3.0".into() };
        assert_eq!(serde_json::to_value(&s).unwrap()["state"], "waitingForRecording");
        let s = Status::UpToDate { checked_at: 1 };
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            serde_json::json!({ "state": "upToDate", "checkedAt": 1 })
        );
    }
}
