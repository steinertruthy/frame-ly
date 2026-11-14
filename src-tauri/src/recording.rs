//! Screen recording flow: target resolution, countdown, native start/pause/resume/stop,
//! status ticker (tray + UI), file delivery, video preview and crash recovery.
//!
//! The native layer encodes straight to a temp MP4 in the app cache. A small manifest next to
//! it marks the recording as in progress; if FrameLy dies, the next launch finds the manifest
//! and moves the (fragmented, still playable) file into the recordings folder.

use crate::core::sounds::Cue;
use crate::{
    core::{
        errors::{AppError, AppResult},
        paths,
        recorder::{format_elapsed, Phase, Recorder},
        video,
    },
    platform::{
        Display, NativeAudio, NativeEvent, NativeRecording, NativeTarget, RecordingStats, Rect, WindowInfo,
    },
    screenshot::{self, PreviewKind, Purpose, SelectMode},
    tray, AppState,
};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const STATUS_EVENT: &str = "recording://status";
pub const FINISHED_EVENT: &str = "recording://finished";
pub const FAILED_EVENT: &str = "recording://failed";
const COUNTDOWN: &str = "countdown";
const CONTROLS: &str = "recbar";
/// How long before capture the start cue begins: exactly its length. No extra slack: the native start itself takes ~200–350 ms before the first audio is
/// captured, which keeps the cue out of the microphone while it still lands right before the
/// recording starts.
fn start_cue_lead() -> Duration {
    crate::core::sounds::duration(Cue::RecordStart)
}
const FRAME: &str = "rec-frame";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Request {
    /// A whole display; `None` = the display under the mouse pointer.
    Display { display_id: Option<String> },
    /// Pick a region or window with the overlay first.
    Select { mode: SelectMode },
}

#[derive(Debug, Clone)]
pub enum Target {
    Display(Display),
    Window {
        window: WindowInfo,
        display: Option<Display>,
    },
    /// `rect` is display-local, in the units of `display.bounds`.
    Region {
        display: Display,
        rect: Rect,
    },
}

impl Target {
    fn kind(&self) -> &'static str {
        match self {
            Target::Display(_) => "display",
            Target::Window { .. } => "window",
            Target::Region { .. } => "region",
        }
    }

    fn display(&self) -> Option<&Display> {
        match self {
            Target::Display(d) | Target::Region { display: d, .. } => Some(d),
            Target::Window { display, .. } => display.as_ref(),
        }
    }

    /// Native pixel size of what will be captured.
    fn source_pixels(&self) -> (u32, u32) {
        let scale =
            |d: Option<&Display>| d.map(|d| d.pixel_width as f64 / d.bounds.width.max(1.0)).unwrap_or(1.0);
        match self {
            Target::Display(d) => (d.pixel_width, d.pixel_height),
            Target::Window { window, display } => {
                let k = scale(display.as_ref());
                ((window.bounds.width * k).round() as u32, (window.bounds.height * k).round() as u32)
            }
            Target::Region { display, rect } => {
                let k = scale(Some(display));
                ((rect.width * k).round() as u32, (rect.height * k).round() as u32)
            }
        }
    }

    fn native(&self) -> NativeTarget {
        let mut t = NativeTarget {
            kind: self.kind(),
            display_id: self.display().map(|d| d.id.clone()),
            window_id: None,
            x: None,
            y: None,
            width: None,
            height: None,
        };
        match self {
            Target::Window { window, .. } => t.window_id = Some(window.id.clone()),
            Target::Region { rect, .. } => {
                (t.x, t.y, t.width, t.height) =
                    (Some(rect.x), Some(rect.y), Some(rect.width), Some(rect.height));
            }
            Target::Display(_) => {}
        }
        t
    }
}

struct Active {
    id: String,
    kind: &'static str,
    /// (system audio, microphone) actually captured.
    audio: Option<(bool, bool)>,
    size: (u32, u32),
    started_at: chrono::DateTime<chrono::Local>,
    temp: PathBuf,
    thumb: PathBuf,
    manifest: PathBuf,
}

#[derive(Default)]
struct Inner {
    recorder: Recorder,
    active: Option<Active>,
}

#[derive(Default)]
pub struct Recordings {
    inner: Mutex<Inner>,
    cancel_countdown: AtomicBool,
    pub last: Mutex<Option<Arc<LastRecording>>>,
    recovered: Mutex<Vec<String>>,
}

pub struct LastRecording {
    pub path: PathBuf,
    pub thumb: Option<PathBuf>,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecordingInfo {
    pub path: String,
    pub file_name: String,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub has_thumbnail: bool,
}

impl LastRecording {
    pub fn info(&self) -> RecordingInfo {
        RecordingInfo {
            path: self.path.display().to_string(),
            file_name: self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
            duration_ms: self.duration_ms,
            width: self.width,
            height: self.height,
            bytes: self.bytes,
            has_thumbnail: self.thumb.as_ref().is_some_and(|t| t.exists()),
        }
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub phase: Phase,
    pub elapsed_ms: u64,
    pub target: Option<&'static str>,
    /// Which audio sources this recording captures, and their current peak levels.
    pub audio: Option<AudioStatus>,
}

#[derive(Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioStatus {
    pub system: bool,
    pub microphone: bool,
    pub levels: crate::platform::AudioLevels,
}

pub const WARNING_EVENT: &str = "recording://warning";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    id: String,
    started_at: String,
    temp: PathBuf,
}

fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

async fn on_worker<T: Send + 'static>(f: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| AppError::Runtime(e.to_string()))?
}

fn work_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app.path().app_cache_dir()?.join("recordings");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn status(app: &AppHandle) -> Status {
    let st = state(app);
    let inner = st.recordings.inner.lock().unwrap();
    let audio = inner.active.as_ref().and_then(|a| a.audio).map(|(system, microphone)| AudioStatus {
        system,
        microphone,
        levels: if matches!(inner.recorder.phase(), Phase::Recording) {
            st.backend.recording_levels()
        } else {
            Default::default()
        },
    });
    Status {
        phase: inner.recorder.phase(),
        elapsed_ms: inner.recorder.elapsed(Instant::now()).as_millis() as u64,
        target: inner.active.as_ref().map(|a| a.kind),
        audio,
    }
}

fn publish(app: &AppHandle) {
    let s = status(app);
    tray::show_recording(app, s.phase, &format_elapsed(Duration::from_millis(s.elapsed_ms)));
    let _ = app.emit(STATUS_EVENT, &s);
}

fn fail(app: &AppHandle, e: &AppError) {
    tracing::warn!(error = %e, "recording failed");
    let _ = app.emit(FAILED_EVENT, e);
}

/// Entry point for UI, tray and CLI.
pub async fn start(app: AppHandle, request: Request) -> AppResult<()> {
    if !state(&app).backend.supports_recording() {
        return Err(AppError::Platform("screen recording is not available on this system yet".into()));
    }
    if state(&app).recordings.inner.lock().unwrap().recorder.is_active() {
        return Err(AppError::Runtime("a recording is already in progress".into()));
    }
    match request {
        Request::Select { mode } => screenshot::select(app, mode, Purpose::Record).await,
        Request::Display { display_id } => {
            let backend = Arc::clone(&state(&app).backend);
            let display = on_worker(move || {
                let id = display_id.or_else(|| backend.display_at_cursor());
                let displays = backend.displays()?;
                displays
                    .iter()
                    .find(|d| Some(&d.id) == id.as_ref())
                    .or_else(|| displays.iter().find(|d| d.is_primary))
                    .cloned()
                    .ok_or_else(|| AppError::Platform("no display found".into()))
            })
            .await?;
            begin(app, Target::Display(display)).await
        }
    }
}

/// Fire-and-forget variant for the tray and CLI; failures surface in the main window.
pub fn trigger(app: &AppHandle, request: Request) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start(app.clone(), request).await {
            crate::windows::show_main(&app);
            fail(&app, &e);
        }
    });
}

/// Countdown (optional), then native start. Called after the target is known.
pub async fn begin(app: AppHandle, target: Target) -> AppResult<()> {
    let settings = state(&app).settings.get();
    let rec = settings.recording.clone();
    state(&app).recordings.inner.lock().unwrap().recorder.begin_countdown()?;
    publish(&app);

    if rec.countdown > 0 {
        let st = state(&app);
        st.recordings.cancel_countdown.store(false, Ordering::SeqCst);
        if let Err(e) = open_countdown(&app, target.display(), rec.countdown) {
            tracing::warn!(error = %e, "countdown window unavailable; continuing without it");
        }
        let deadline = Instant::now() + Duration::from_secs(rec.countdown as u64);
        // The start cue rings out during the last moments of the countdown, so it is over
        // before capture begins (otherwise the microphone records it).
        let cue_at = deadline.checked_sub(start_cue_lead()).unwrap_or(deadline);
        let mut cued = false;
        while Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(20)).await;
            if !cued && Instant::now() >= cue_at {
                cued = true;
                crate::notify::cue(&app, Cue::RecordStart);
            }
            if st.recordings.cancel_countdown.load(Ordering::SeqCst) {
                close_window(&app, COUNTDOWN);
                st.recordings.inner.lock().unwrap().recorder.reset();
                publish(&app);
                tracing::info!("recording cancelled during countdown");
                return Ok(());
            }
        }
        close_window(&app, COUNTDOWN);
    } else if crate::notify::cue(&app, Cue::RecordStart) {
        tokio::time::sleep(start_cue_lead()).await;
    }

    // Windows excludes FrameLy from captures window by window (macOS excludes the whole app),
    // so the main window must be gone before the first frame.
    #[cfg(windows)]
    crate::windows::hide_main(&app);

    match start_native(&app, &target, &rec).await {
        Ok(()) => {
            if let Target::Region { display, rect } = &target {
                if let Err(e) = open_region_frame(&app, display, rect) {
                    tracing::debug!(error = %e, "region frame unavailable");
                }
            }
            crate::windows::hide_main(&app);
            if rec.show_controls {
                if let Err(e) = open_controls(&app, target.display()) {
                    tracing::warn!(error = %e, "recording controls unavailable");
                }
            }
            spawn_ticker(app.clone());
            Ok(())
        }
        Err(e) => {
            state(&app).recordings.inner.lock().unwrap().recorder.reset();
            publish(&app);
            #[cfg(windows)]
            crate::windows::show_main(&app);
            Err(e)
        }
    }
}

async fn start_native(
    app: &AppHandle,
    target: &Target,
    rec: &crate::core::settings::Recording,
) -> AppResult<()> {
    let id = chrono::Local::now().format("%Y%m%d-%H%M%S-%3f").to_string();
    let dir = work_dir(app)?;
    let temp = dir.join(format!("{id}.mp4"));
    let thumb = dir.join(format!("{id}.jpg"));
    let manifest = dir.join(format!("{id}.json"));
    let started_at = chrono::Local::now();

    let audio = resolve_audio(app).await;
    let size = video::output_size(target.source_pixels(), rec.resolution, rec.codec);
    let config = NativeRecording {
        target: target.native(),
        audio: audio.clone(),
        width: size.0,
        height: size.1,
        fps: rec.fps,
        bitrate: video::bitrate(size, rec.fps, rec.quality, rec.codec),
        quality: video::quality_level(rec.quality),
        codec: rec.codec,
        shows_cursor: rec.show_cursor,
        output_path: temp.display().to_string(),
        thumbnail_path: thumb.display().to_string(),
    };
    std::fs::write(
        &manifest,
        serde_json::to_vec(&Manifest {
            id: id.clone(),
            started_at: started_at.to_rfc3339(),
            temp: temp.clone(),
        })?,
    )?;

    let events_app = app.clone();
    let events: crate::platform::RecordingEvents = Arc::new(move |event| match event {
        NativeEvent::Started => tracing::debug!("first frame recorded"),
        NativeEvent::Stopped { reason, duration_ms, frames } => {
            tracing::warn!(%reason, duration_ms, frames, "recording stopped by the system");
            let app = events_app.clone();
            tauri::async_runtime::spawn(async move {
                let stats = RecordingStats { duration_ms, frames, width: 0, height: 0, audio: false };
                finish(app, Some(stats)).await;
            });
        }
    });

    let backend = Arc::clone(&state(app).backend);
    let cfg = config.clone();
    let t = Instant::now();
    let started = on_worker(move || backend.start_recording(&cfg, events)).await;
    if let Err(e) = started {
        let _ = std::fs::remove_file(&manifest);
        return Err(e);
    }
    tracing::info!(
        ms = t.elapsed().as_millis() as u64,
        target = target.kind(),
        width = config.width,
        height = config.height,
        fps = config.fps,
        bitrate = config.bitrate,
        codec = ?config.codec,
        "recording started"
    );
    let st = state(app);
    let mut inner = st.recordings.inner.lock().unwrap();
    inner.recorder.started(Instant::now())?;
    inner.active = Some(Active {
        id,
        kind: target.kind(),
        audio: audio.map(|a| (a.system, a.microphone)),
        size: (config.width, config.height),
        started_at,
        temp,
        thumb,
        manifest,
    });
    Ok(())
}

/// Turns audio settings into what can actually be captured right now. The microphone needs
/// permission (asked here the first time) and a present device; when either is missing the
/// recording proceeds without it — or with the default input — and the UI is warned.
async fn resolve_audio(app: &AppHandle) -> Option<NativeAudio> {
    use crate::platform::{Permission, PermissionState};
    let a = state(app).settings.get().audio;
    let mut microphone = a.microphone;
    let mut device = a.microphone_device.clone();
    if microphone {
        let backend = Arc::clone(&state(app).backend);
        let (perm, inputs) = on_worker(move || {
            let mut p = backend.permissions().microphone;
            if p == PermissionState::NotDetermined {
                p = backend.request_permission(Permission::Microphone)?;
            }
            Ok((p, backend.audio_inputs().unwrap_or_default()))
        })
        .await
        .unwrap_or((PermissionState::Denied, Vec::new()));
        if perm != PermissionState::Granted {
            microphone = false;
            warn(app, "microphone_permission");
        } else if inputs.is_empty() {
            microphone = false;
            warn(app, "microphone_missing");
        } else if let Some(id) = &device {
            if !inputs.iter().any(|d| &d.id == id) {
                device = None;
                warn(app, "microphone_fallback");
            }
        }
    }
    (a.system_audio || microphone).then_some(NativeAudio {
        system: a.system_audio,
        microphone,
        microphone_id: device,
        system_gain: a.system_gain,
        microphone_gain: a.microphone_gain,
    })
}

fn warn(app: &AppHandle, code: &str) {
    tracing::warn!(code, "recording audio warning");
    let _ = app.emit(WARNING_EVENT, code);
}

/// Emits the status four times a second while a recording is active (tray title, UI timer, meters).
fn spawn_ticker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let phase = state(&app).recordings.inner.lock().unwrap().recorder.phase();
            if !matches!(phase, Phase::Recording | Phase::Paused) {
                break;
            }
            publish(&app);
            // 4 Hz: smooth enough for level meters, negligible cost.
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    });
}

pub fn pause(app: &AppHandle) -> AppResult<()> {
    state(app).recordings.inner.lock().unwrap().recorder.pause(Instant::now())?;
    state(app).backend.pause_recording();
    tracing::info!("recording paused");
    publish(app);
    Ok(())
}

pub fn resume(app: &AppHandle) -> AppResult<()> {
    state(app).recordings.inner.lock().unwrap().recorder.resume(Instant::now())?;
    state(app).backend.resume_recording();
    tracing::info!("recording resumed");
    publish(app);
    Ok(())
}

pub fn toggle_pause(app: &AppHandle) -> AppResult<()> {
    match status(app).phase {
        Phase::Paused => resume(app),
        _ => pause(app),
    }
}

/// Cancels a running countdown (no-op otherwise).
pub fn cancel_countdown(app: &AppHandle) {
    state(app).recordings.cancel_countdown.store(true, Ordering::SeqCst);
}

pub async fn stop(app: AppHandle) -> AppResult<()> {
    match status(&app).phase {
        Phase::Countdown => {
            cancel_countdown(&app);
            Ok(())
        }
        Phase::Recording | Phase::Paused => {
            finish(app, None).await;
            Ok(())
        }
        p => Err(AppError::Runtime(format!("nothing to stop ({p:?})"))),
    }
}

/// Finalizes the file and delivers it. `system_stats` is set when the OS already stopped and
/// finalized the recording.
async fn finish(app: AppHandle, system_stats: Option<RecordingStats>) {
    {
        let st = state(&app);
        let mut inner = st.recordings.inner.lock().unwrap();
        if inner.recorder.finalizing(Instant::now()).is_err() {
            return; // already finishing (e.g. user stop raced with a system stop)
        }
    }
    publish(&app);
    close_window(&app, FRAME);
    close_window(&app, CONTROLS);

    let backend = Arc::clone(&state(&app).backend);
    let stats = match system_stats {
        Some(s) => {
            backend.discard_recording();
            Ok(s)
        }
        None => on_worker(move || backend.stop_recording()).await,
    };
    // After the native stop, so the cue can't leak into the file via loopback audio.
    crate::notify::cue(&app, Cue::RecordStop);
    let active = state(&app).recordings.inner.lock().unwrap().active.take();
    let result = match (stats, active) {
        (Ok(stats), Some(active)) => deliver(&app, active, stats).await,
        (Err(e), Some(active)) => {
            cleanup(&active);
            Err(e)
        }
        (_, None) => Err(AppError::Runtime("recording state lost".into())),
    };
    state(&app).recordings.inner.lock().unwrap().recorder.reset();
    publish(&app);
    if let Err(e) = result {
        if !matches!(e, AppError::EmptyRecording) {
            crate::windows::show_main(&app);
        }
        fail(&app, &e);
    }
}

fn cleanup(active: &Active) {
    let _ = std::fs::remove_file(&active.temp);
    let _ = std::fs::remove_file(&active.thumb);
    let _ = std::fs::remove_file(&active.manifest);
}

/// Moves a file, falling back to copy + delete across volumes.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    std::fs::copy(from, to)?;
    std::fs::remove_file(from)
}

async fn deliver(app: &AppHandle, active: Active, stats: RecordingStats) -> AppResult<()> {
    let dir = state(app).settings.get().storage.recordings_dir;
    let thumbs = app.path().app_cache_dir()?.join("thumbs");
    let started = active.started_at;
    let last = on_worker(move || {
        std::fs::create_dir_all(&dir)?;
        std::fs::create_dir_all(&thumbs)?;
        let dest = paths::unique_path(&dir, &paths::capture_file_name(&started, "mp4"));
        move_file(&active.temp, &dest)?;
        let thumb = thumbs.join(format!("{}.jpg", active.id));
        let thumb = move_file(&active.thumb, &thumb).ok().map(|_| thumb);
        let _ = std::fs::remove_file(&active.manifest);
        let bytes = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
        Ok(LastRecording {
            path: dest,
            thumb,
            duration_ms: stats.duration_ms,
            width: active.size.0,
            height: active.size.1,
            bytes,
        })
    })
    .await?;
    let info = last.info();
    tracing::info!(path = %info.path, duration_ms = info.duration_ms, bytes = info.bytes, frames = stats.frames, "recording saved");
    crate::library::add(
        app,
        &last.path,
        crate::core::library::Meta {
            width: Some(last.width),
            height: Some(last.height),
            duration_ms: Some(last.duration_ms),
        },
    );
    *state(app).recordings.last.lock().unwrap() = Some(Arc::new(last));
    app.emit(FINISHED_EVENT, &info)?;
    let settings = state(app).settings.get();
    if settings.recording.show_preview {
        screenshot::show_preview(app, PreviewKind::Video)?;
    } else {
        let s = crate::core::locale::tray_strings(crate::core::locale::current(settings.general.language));
        crate::notify::send(app, s.recording_saved, &info.file_name);
    }
    Ok(())
}

/// On launch: moves recordings left behind by a crash into the recordings folder.
pub fn recover(app: &AppHandle) {
    let Ok(dir) = work_dir(app) else { return };
    let dest_dir = state(app).settings.get().storage.recordings_dir;
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut recovered = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(manifest) = std::fs::read(&path)
            .map_err(AppError::from)
            .and_then(|b| Ok(serde_json::from_slice::<Manifest>(&b)?))
        else {
            let _ = std::fs::remove_file(&path);
            continue;
        };
        let usable = std::fs::metadata(&manifest.temp).map(|m| m.len() > 0).unwrap_or(false);
        if usable {
            let started = chrono::DateTime::parse_from_rfc3339(&manifest.started_at)
                .map(|d| d.with_timezone(&chrono::Local))
                .unwrap_or_else(|_| chrono::Local::now());
            let name = paths::capture_file_name(&started, "mp4").replace(".mp4", " (recovered).mp4");
            let _ = std::fs::create_dir_all(&dest_dir);
            let dest = paths::unique_path(&dest_dir, &name);
            match move_file(&manifest.temp, &dest) {
                Ok(()) => {
                    tracing::warn!(path = %dest.display(), "recovered an interrupted recording");
                    recovered.push(dest.display().to_string());
                }
                Err(e) => tracing::error!(error = %e, "could not recover recording"),
            }
        } else {
            let _ = std::fs::remove_file(&manifest.temp);
        }
        let _ = std::fs::remove_file(dir.join(format!("{}.jpg", manifest.id)));
        let _ = std::fs::remove_file(&path);
    }
    *state(app).recordings.recovered.lock().unwrap() = recovered;
}

/// Recordings recovered at launch (reported once to the UI).
pub fn take_recovered(app: &AppHandle) -> Vec<String> {
    std::mem::take(&mut *state(app).recordings.recovered.lock().unwrap())
}

fn close_window(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.destroy();
    }
}

/// Logical (Tauri) coordinates of a rect in a display's OS units.
fn to_logical(display: &Display, r: Rect) -> Rect {
    let s = if cfg!(windows) { display.scale_factor } else { 1.0 };
    Rect { x: r.x / s, y: r.y / s, width: r.width / s, height: r.height / s }
}

fn open_countdown(app: &AppHandle, display: Option<&Display>, seconds: u32) -> AppResult<()> {
    const SIZE: f64 = 220.0;
    let center = display
        .map(|d| to_logical(d, d.bounds))
        .map(|b| (b.x + b.width / 2.0, b.y + b.height / 2.0))
        .unwrap_or((600.0, 400.0));
    let window = WebviewWindowBuilder::new(
        app,
        COUNTDOWN,
        WebviewUrl::App(format!("countdown.html?n={seconds}").into()),
    )
    .title("FrameLy")
    .inner_size(SIZE, SIZE)
    .position(center.0 - SIZE / 2.0, center.1 - SIZE / 2.0)
    .decorations(false)
    .transparent(true)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .focused(true)
    .visible_on_all_workspaces(true)
    .build()?;
    crate::platform::chrome::configure_floating(&window);
    Ok(())
}

/// Floating recording controls, bottom-centre of the recorded display's work area (above the
/// Dock / taskbar). Never focused, so it doesn't steal keyboard input from what's recorded;
/// excluded from the video like every FrameLy window.
fn open_controls(app: &AppHandle, display: Option<&Display>) -> AppResult<()> {
    const W: f64 = 340.0;
    const H: f64 = 60.0;
    const MARGIN: f64 = 18.0;
    let display = match display {
        Some(d) => d.clone(),
        None => state(app)
            .backend
            .displays()?
            .into_iter()
            .find(|d| d.is_primary)
            .ok_or_else(|| AppError::Runtime("no display for the recording controls".into()))?,
    };
    let area = to_logical(&display, display.work_area);
    let window = WebviewWindowBuilder::new(app, CONTROLS, WebviewUrl::App("recbar.html".into()))
        .title("FrameLy")
        .inner_size(W, H)
        .position(area.x + (area.width - W) / 2.0, area.y + area.height - H - MARGIN)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible_on_all_workspaces(true)
        .build()?;
    crate::platform::chrome::configure_floating(&window);
    Ok(())
}

/// Thin click-through border around a recorded region (excluded from the recording itself).
fn open_region_frame(app: &AppHandle, display: &Display, rect: &Rect) -> AppResult<()> {
    const PAD: f64 = 4.0;
    let d = to_logical(display, display.bounds);
    let r = to_logical(display, *rect);
    let window = WebviewWindowBuilder::new(app, FRAME, WebviewUrl::App("frame.html".into()))
        .title("FrameLy")
        .inner_size(r.width + PAD * 2.0, r.height + PAD * 2.0)
        .position(d.x + r.x - PAD, d.y + r.y - PAD)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible_on_all_workspaces(true)
        .build()?;
    window.set_ignore_cursor_events(true)?;
    crate::platform::chrome::configure_floating(&window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retina() -> Display {
        Display {
            id: "1".into(),
            name: "Built-in".into(),
            bounds: Rect { x: 0.0, y: 0.0, width: 1512.0, height: 982.0 },
            work_area: Rect { x: 0.0, y: 33.0, width: 1512.0, height: 870.0 },
            pixel_width: 3024,
            pixel_height: 1964,
            scale_factor: 2.0,
            refresh_rate: Some(120.0),
            is_primary: true,
        }
    }

    #[test]
    fn source_pixels_follow_display_scale() {
        assert_eq!(Target::Display(retina()).source_pixels(), (3024, 1964));
        let region = Target::Region {
            display: retina(),
            rect: Rect { x: 10.0, y: 10.0, width: 400.5, height: 300.0 },
        };
        assert_eq!(region.source_pixels(), (801, 600));
        let window = WindowInfo {
            id: "9".into(),
            title: "Doc".into(),
            app_name: "Pages".into(),
            app_id: None,
            pid: 1,
            bounds: Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        };
        let t = Target::Window { window: window.clone(), display: Some(retina()) };
        assert_eq!(t.source_pixels(), (1600, 1200));
        assert_eq!(Target::Window { window, display: None }.source_pixels(), (800, 600));
    }

    #[test]
    fn native_target_carries_region_rect() {
        let t = Target::Region { display: retina(), rect: Rect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 } }
            .native();
        assert_eq!((t.kind, t.display_id.as_deref()), ("region", Some("1")));
        assert_eq!((t.x, t.y, t.width, t.height), (Some(1.0), Some(2.0), Some(3.0), Some(4.0)));
    }
}
