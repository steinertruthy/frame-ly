//! FrameLy AI use cases: transcription, summaries and topics on top of the media decoder,
//! speech recognition, the local language-model runtime, the model manager and the provider
//! layer. The UI talks to this module through `commands::intelligence`; it never reaches the
//! runtimes directly.
//!
//! Work runs one job at a time on a background thread (transcription and generation compete
//! for the same CPU/GPU), with real progress events (`ai://job`), cancellation, and resumption
//! of interrupted transcriptions.

pub mod catalog;
pub mod estimate;
pub mod generation;
pub mod hardware;
pub mod llm;
pub mod models;
pub mod net;
pub mod providers;
pub mod speech;

use crate::{
    core::{
        errors::{AppError, AppResult},
        settings::{Detail, Intelligence as Prefs, ProviderId},
        transcript::{self, Segment},
        transcripts::{Generated, MediaRef, Status, Transcript, Transcripts},
    },
    AppState,
};
use generation::Task;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Instant, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const JOB_EVENT: &str = "ai://job";
pub const MODELS_EVENT: &str = "ai://models";
pub const TRANSCRIPT_EVENT: &str = "ai://transcript";
pub const NOTICE_EVENT: &str = "ai://notice";

/// Media FrameLy can transcribe, per platform (what the native decoders read).
pub fn importable_extensions() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["mp4", "mov", "m4v", "m4a", "mp3", "wav", "aac", "flac", "aif", "aiff", "caf"]
    } else {
        &["mp4", "mov", "m4v", "m4a", "mp3", "wav", "aac", "flac", "wma"]
    }
}

fn is_video(path: &Path) -> bool {
    matches!(path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(), Some("mp4" | "mov" | "m4v"))
}

#[derive(Debug, Clone)]
enum JobKind {
    Transcribe { precision: String, language: String },
    Generate { task: Task, provider: ProviderId, detail: Detail },
}

#[derive(Debug, Clone)]
struct Job {
    id: u64,
    transcript_id: i64,
    kind: JobKind,
    cancel: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub id: u64,
    pub transcript_id: i64,
    /// "transcribe" | "summary" | "topics"
    pub kind: &'static str,
    /// "queued" | "running" | "done" | "failed" | "cancelled"
    pub state: &'static str,
    /// "queued" | "preparing" | "loading" | "transcribing" | "generating" | "combining" | "saving"
    pub phase: &'static str,
    /// 0…1 when measurable; None shows an activity indicator.
    pub progress: Option<f64>,
    /// Remaining time from the measured throughput of this run.
    pub eta_ms: Option<u64>,
    /// Part n of m (long transcripts split for the model's context).
    pub part: Option<(usize, usize)>,
    pub provider: Option<ProviderId>,
    pub error: Option<JobError>,
}

#[derive(Default)]
pub struct Intelligence {
    db: Mutex<Option<Transcripts>>,
    models: OnceLock<Arc<models::ModelManager>>,
    hardware: OnceLock<hardware::Hardware>,
    queue: Mutex<VecDeque<Job>>,
    current: Mutex<Option<Job>>,
    statuses: Mutex<Vec<JobStatus>>,
    worker: AtomicBool,
    next_id: AtomicU64,
    temp_dir: OnceLock<PathBuf>,
}

fn st<R: Runtime>(app: &AppHandle<R>) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

pub fn prefs<R: Runtime>(app: &AppHandle<R>) -> Prefs {
    st(app).settings.get().intelligence
}

pub fn with_db<R: Runtime, T>(app: &AppHandle<R>, f: impl FnOnce(&mut Transcripts) -> AppResult<T>) -> AppResult<T> {
    let state = st(app);
    let mut g = state.intelligence.db.lock().unwrap();
    let db = g.as_mut().ok_or_else(|| AppError::Runtime("transcripts not ready".into()))?;
    f(db)
}

pub fn models<R: Runtime>(app: &AppHandle<R>) -> Arc<models::ModelManager> {
    st(app).intelligence.models.get().expect("intelligence initialized").clone()
}

pub fn hardware<R: Runtime>(app: &AppHandle<R>) -> hardware::Hardware {
    st(app).intelligence.hardware.get_or_init(hardware::detect).clone()
}

pub fn profile<R: Runtime>(app: &AppHandle<R>) -> crate::core::settings::AiProfile {
    hardware::resolve(prefs(app).profile, &hardware(app))
}

fn temp_dir<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    st(app).intelligence.temp_dir.get().cloned().unwrap_or_else(std::env::temp_dir)
}

/// Opens the store, marks jobs that were running when FrameLy last quit as interrupted, and
/// wires model-manager events.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> AppResult<()> {
    // whisper.cpp/ggml print diagnostics to stderr; keep them out of the app's output.
    whisper_rs::install_logging_hooks();
    let data = app.path().app_data_dir()?;
    let db = Transcripts::open(&data.join("transcripts.sqlite3"))?;
    let n = db.mark_interrupted()?;
    if n > 0 {
        tracing::info!(count = n, "transcriptions interrupted by the previous session");
    }
    let state = st(app);
    *state.intelligence.db.lock().unwrap() = Some(db);
    let mgr = Arc::new(models::ModelManager::new(data.join("models")));
    let handle = app.clone();
    mgr.set_notify(Arc::new(move |s| {
        let _ = handle.emit(MODELS_EVENT, s);
    }));
    let _ = state.intelligence.models.set(mgr);
    let tmp = app.path().app_cache_dir()?.join("intelligence");
    std::fs::create_dir_all(&tmp)?;
    let _ = state.intelligence.temp_dir.set(tmp);
    // Hardware probing (DXGI, sysctl) off the startup path.
    let handle = app.clone();
    std::thread::spawn(move || {
        hardware(&handle);
    });
    // Imported files outside the media folders stay playable.
    if let Ok(list) = with_db(app, |db| db.list()) {
        for t in list {
            let _ = app.asset_protocol_scope().allow_file(&t.media.path);
        }
    }
    Ok(())
}

/// Facts about a media file before transcribing it.
pub fn probe<R: Runtime>(app: &AppHandle<R>, path: &Path) -> AppResult<MediaRef> {
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).unwrap_or_default();
    if !importable_extensions().contains(&ext.as_str()) {
        return Err(AppError::Unsupported(ext));
    }
    let md = std::fs::metadata(path).map_err(|_| AppError::NotFound)?;
    let mtime = md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0);
    let meta = st(app).backend.video_info(path).unwrap_or_default();
    Ok(MediaRef {
        path: path.to_string_lossy().into_owned(),
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        kind: if is_video(path) { "video".into() } else { "audio".into() },
        size: md.len(),
        mtime,
        duration_ms: meta.duration_ms.unwrap_or(0),
    })
}

fn emit_status<R: Runtime>(app: &AppHandle<R>, s: JobStatus) {
    {
        let state = st(app);
        let mut list = state.intelligence.statuses.lock().unwrap();
        list.retain(|x| x.id != s.id);
        // Keep finished ones briefly so a page opened later still sees the outcome.
        list.push(s.clone());
        if list.len() > 50 {
            list.remove(0);
        }
    }
    let _ = app.emit(JOB_EVENT, s);
}

pub fn jobs<R: Runtime>(app: &AppHandle<R>) -> Vec<JobStatus> {
    st(app).intelligence.statuses.lock().unwrap().clone()
}

fn kind_name(k: &JobKind) -> &'static str {
    match k {
        JobKind::Transcribe { .. } => "transcribe",
        JobKind::Generate { task, .. } => task.as_str(),
    }
}

fn base_status(job: &Job, state: &'static str, phase: &'static str) -> JobStatus {
    JobStatus {
        id: job.id,
        transcript_id: job.transcript_id,
        kind: kind_name(&job.kind),
        state,
        phase,
        progress: None,
        eta_ms: None,
        part: None,
        provider: match job.kind {
            JobKind::Generate { provider, .. } => Some(provider),
            _ => None,
        },
        error: None,
    }
}

fn enqueue<R: Runtime>(app: &AppHandle<R>, transcript_id: i64, kind: JobKind) -> u64 {
    let state = st(app);
    let intel = &state.intelligence;
    let id = intel.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let job = Job { id, transcript_id, kind, cancel: Arc::new(AtomicBool::new(false)) };
    emit_status(app, base_status(&job, "queued", "queued"));
    intel.queue.lock().unwrap().push_back(job);
    if !intel.worker.swap(true, Ordering::SeqCst) {
        let app = app.clone();
        std::thread::Builder::new()
            .name("framely-ai".into())
            .spawn(move || worker_loop(app))
            .expect("spawn AI worker");
    }
    id
}

fn worker_loop<R: Runtime>(app: AppHandle<R>) {
    loop {
        let job = {
            let state = st(&app);
            let mut q = state.intelligence.queue.lock().unwrap();
            match q.pop_front() {
                Some(j) => j,
                None => {
                    state.intelligence.worker.store(false, Ordering::SeqCst);
                    return;
                }
            }
        };
        *st(&app).intelligence.current.lock().unwrap() = Some(job.clone());
        let result = match &job.kind {
            JobKind::Transcribe { .. } => run_transcription(&app, &job),
            JobKind::Generate { task, provider, detail } => run_generation(&app, &job, *task, *provider, *detail),
        };
        *st(&app).intelligence.current.lock().unwrap() = None;
        match result {
            Ok(()) => emit_status(&app, JobStatus { progress: Some(1.0), ..base_status(&job, "done", "saving") }),
            Err(AppError::Cancelled) => emit_status(&app, base_status(&job, "cancelled", "queued")),
            Err(e) => {
                tracing::warn!(job = job.id, error = %e, "AI job failed");
                emit_status(
                    &app,
                    JobStatus { error: Some(JobError { code: e.code().into(), message: e.to_string() }), ..base_status(&job, "failed", "queued") },
                );
            }
        }
    }
}

pub fn cancel<R: Runtime>(app: &AppHandle<R>, job_id: u64) {
    let state = st(app);
    let intel = &state.intelligence;
    let queued = {
        let mut q = intel.queue.lock().unwrap();
        let pos = q.iter().position(|j| j.id == job_id);
        pos.and_then(|p| q.remove(p))
    };
    if let Some(job) = queued {
        if let JobKind::Transcribe { .. } = job.kind {
            let _ = with_db(app, |db| db.set_status(job.transcript_id, Status::Cancelled, None));
        }
        emit_status(app, base_status(&job, "cancelled", "queued"));
        return;
    }
    let current = intel.current.lock().unwrap().clone();
    if let Some(job) = current.filter(|j| j.id == job_id) {
        job.cancel.store(true, Ordering::Relaxed);
    }
}

/// Is a job for this transcript queued or running?
fn busy<R: Runtime>(app: &AppHandle<R>, transcript_id: i64, kind: &str) -> Option<u64> {
    let state = st(app);
    let intel = &state.intelligence;
    let mut all: Vec<Job> = intel.current.lock().unwrap().clone().into_iter().collect();
    all.extend(intel.queue.lock().unwrap().iter().cloned());
    all.into_iter().find(|j| j.transcript_id == transcript_id && kind_name(&j.kind) == kind).map(|j| j.id)
}

// ---------------------------------------------------------------------------------------------
// Transcription

/// Starts (or resumes) the transcription of a media file. A finished transcript of the same
/// file with the same settings is reused instead of processing it again.
pub fn start_transcription<R: Runtime>(app: &AppHandle<R>, path: &Path, precision: Option<String>, language: Option<String>) -> AppResult<(Transcript, Option<u64>)> {
    let p = prefs(app);
    if !hardware(app).supported {
        return Err(AppError::Unsupported("cpu".into()));
    }
    let precision = catalog::precision_for(profile(app), catalog::parse_precision(precision.as_deref().unwrap_or("auto")));
    let spec = catalog::speech_model(precision);
    if models(app).ready_path(spec).is_none() {
        return Err(AppError::ModelMissing(spec.id.into()));
    }
    let language = language.unwrap_or(p.language);
    let media = probe(app, path)?;
    let _ = app.asset_protocol_scope().allow_file(path);
    let t = with_db(app, |db| db.begin(&media, &language, catalog::precision_name(precision), spec.id))?;
    if t.status == Status::Done {
        return Ok((t, None));
    }
    if let Some(job) = busy(app, t.id, "transcribe") {
        return Ok((t, Some(job)));
    }
    let job = enqueue(app, t.id, JobKind::Transcribe { precision: catalog::precision_name(precision).into(), language });
    Ok((t, Some(job)))
}

fn pcm_paths<R: Runtime>(app: &AppHandle<R>, id: i64) -> (PathBuf, PathBuf) {
    let dir = temp_dir(app);
    (dir.join(format!("{id}.pcm")), dir.join(format!("{id}.pcm.ok")))
}

/// Progress reporter with an ETA from the measured throughput of the current phase.
struct Meter {
    started: Instant,
    from: f64,
}

impl Meter {
    fn new(at: f64) -> Self {
        Self { started: Instant::now(), from: at }
    }
    fn eta(&self, p: f64) -> Option<u64> {
        let done = p - self.from;
        let secs = self.started.elapsed().as_secs_f64();
        (done > 0.02 && secs > 3.0).then(|| (secs / done * (1.0 - p) * 1000.0) as u64)
    }
}

fn run_transcription<R: Runtime>(app: &AppHandle<R>, job: &Job) -> AppResult<()> {
    let JobKind::Transcribe { precision, language } = &job.kind else { unreachable!() };
    let t = with_db(app, |db| db.get(job.transcript_id))?;
    with_db(app, |db| db.set_status(t.id, Status::Running, None))?;
    let result = transcribe_inner(app, job, &t, precision, language);
    let (pcm, ok) = pcm_paths(app, t.id);
    match &result {
        Ok(()) => {
            with_db(app, |db| db.set_status(t.id, Status::Done, None))?;
            let _ = std::fs::remove_file(&pcm);
            let _ = std::fs::remove_file(&ok);
        }
        Err(AppError::Cancelled) => {
            with_db(app, |db| db.set_status(t.id, Status::Cancelled, None))?;
            let _ = std::fs::remove_file(&pcm);
            let _ = std::fs::remove_file(&ok);
        }
        Err(e) => {
            with_db(app, |db| db.set_status(t.id, Status::Failed, Some(e.code())))?;
        }
    }
    let _ = app.emit(TRANSCRIPT_EVENT, t.id);
    if result.is_ok() {
        auto_generate(app, t.id);
    }
    result
}

fn transcribe_inner<R: Runtime>(app: &AppHandle<R>, job: &Job, t: &Transcript, precision: &str, language: &str) -> AppResult<()> {
    let precision = catalog::parse_precision(precision);
    let spec = catalog::speech_model(precision);
    let mgr = models(app);
    let model = mgr.ready_path(spec).ok_or_else(|| AppError::ModelMissing(spec.id.into()))?;
    let _in_use = mgr.use_model(spec);

    // 1. Audio → 16 kHz mono PCM (reused when an interrupted job left it complete).
    let (pcm, ok) = pcm_paths(app, t.id);
    let samples = match std::fs::read_to_string(&ok).ok().and_then(|s| s.trim().parse::<u64>().ok()) {
        Some(n) if std::fs::metadata(&pcm).map(|m| m.len() == n * 4).unwrap_or(false) => n,
        _ => {
            let meter = Meter::new(0.0);
            let mut report = |f: f64| {
                emit_status(app, JobStatus { progress: Some(f), eta_ms: meter.eta(f), ..base_status(job, "running", "preparing") });
            };
            report(0.0);
            let n = crate::platform::decode_audio(Path::new(&t.media.path), &pcm, &mut report, &job.cancel)?;
            if n < speech::RATE as u64 / 2 {
                return Err(AppError::NoAudio);
            }
            std::fs::write(&ok, n.to_string())?;
            n
        }
    };

    // 2. Model + blocks.
    emit_status(app, base_status(job, "running", "loading"));
    let start_block = t.blocks_done;
    let previous_text = if start_block > 0 {
        with_db(app, |db| db.segments(t.id))?.iter().rev().take(6).rev().map(|s| s.text.clone()).collect::<Vec<_>>().join(" ")
    } else {
        String::new()
    };
    let lang = if language == "auto" { t.detected_language.clone().filter(|_| start_block > 0) } else { Some(language.to_string()) };
    let hw = hardware(app);
    let started = Instant::now();
    let from = t.done_ms as f64 / t.media.duration_ms.max(1) as f64;
    let meter = Arc::new(Mutex::new(Meter::new(from)));
    let (app2, job2, meter2) = (app.clone(), job.clone(), meter.clone());
    let last = Arc::new(Mutex::new(Instant::now() - std::time::Duration::from_secs(1)));
    let on_progress: Arc<dyn Fn(f64) + Send + Sync> = Arc::new(move |p: f64| {
        let mut l = last.lock().unwrap();
        if l.elapsed().as_millis() < 250 && p < 1.0 {
            return;
        }
        *l = Instant::now();
        let eta = meter2.lock().unwrap().eta(p);
        emit_status(&app2, JobStatus { progress: Some(p), eta_ms: eta, ..base_status(&job2, "running", "transcribing") });
    });
    let meter3 = meter.clone();
    let progress_loaded = on_progress.clone();
    speech::run(
        speech::Request {
            pcm: &pcm,
            samples,
            model: &model,
            precision,
            language: lang,
            start_block,
            previous_text,
            gpu: hw.accel == "metal",
            cancel: job.cancel.clone(),
        },
        move || {
            // Throughput is measured from here (model loading excluded).
            *meter3.lock().unwrap() = Meter::new(from);
            progress_loaded(from);
        },
        on_progress,
        |b| {
            with_db(app, |db| db.append_block(t.id, b.index, &b.segments, b.done_ms, b.language.as_deref()))?;
            let _ = app.emit(TRANSCRIPT_EVENT, t.id);
            Ok(())
        },
    )?;
    let audio_secs = (samples as f64 / speech::RATE as f64) * (1.0 - from);
    let _ = with_db(app, |db| db.record_perf("asr", spec.id, hw.accel, audio_secs, started.elapsed().as_millis() as u64));
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Summaries and topics

/// Can `provider` run now without asking the user anything? (Used for automatic generation:
/// never sends anything out without prior consent, never falls back to another provider.)
pub fn provider_ready<R: Runtime>(app: &AppHandle<R>, provider: ProviderId) -> Result<(), &'static str> {
    let p = prefs(app);
    match provider {
        ProviderId::Framely => {
            if !hardware(app).supported {
                return Err("unsupported_cpu");
            }
            let spec = catalog::language_model(profile(app));
            models(app).ready_path(spec).map(|_| ()).ok_or("model_missing")
        }
        ProviderId::Apple => apple_reason(None, &p).map_or(Ok(()), Err),
        ext => {
            let key = crate::platform::keystore::get(providers::key_account(ext)).ok().flatten().is_some();
            providers::external_reason(&p, ext, key).map_or(Ok(()), Err)
        }
    }
}

#[cfg(target_os = "macos")]
fn apple_reason(language: Option<&str>, p: &Prefs) -> Option<&'static str> {
    if !p.apple_enabled {
        return Some("disabled");
    }
    match crate::platform::apple_ai::status(language) {
        Ok(s) if s.available && s.supports_language != Some(false) => None,
        Ok(s) if s.available => Some("language"),
        Ok(s) => Some(match s.reason.as_deref() {
            Some("device_not_eligible") => "device_not_eligible",
            Some("not_enabled") => "not_enabled",
            Some("model_not_ready") => "model_not_ready",
            _ => "os_version",
        }),
        Err(_) => Some("os_version"),
    }
}

#[cfg(not(target_os = "macos"))]
fn apple_reason(_: Option<&str>, _: &Prefs) -> Option<&'static str> {
    Some("unsupported")
}

fn auto_generate<R: Runtime>(app: &AppHandle<R>, transcript_id: i64) {
    let p = prefs(app);
    for (task, on, provider, detail) in
        [(Task::Summary, p.summary_auto, p.summary_provider, p.summary_detail), (Task::Topics, p.topics_auto, p.topics_provider, p.topics_detail)]
    {
        if !on {
            continue;
        }
        match provider_ready(app, provider) {
            Ok(()) => {
                enqueue(app, transcript_id, JobKind::Generate { task, provider, detail });
            }
            Err(reason) => {
                let _ = app.emit(NOTICE_EVENT, serde_json::json!({"code": "auto_skipped", "task": task.as_str(), "provider": provider, "reason": reason}));
            }
        }
    }
}

pub fn start_generation<R: Runtime>(app: &AppHandle<R>, transcript_id: i64, task: Task, provider: ProviderId, detail: Detail) -> AppResult<u64> {
    let t = with_db(app, |db| db.get(transcript_id))?;
    if t.status != Status::Done {
        return Err(AppError::Runtime("transcription not finished".into()));
    }
    if let Some(job) = busy(app, transcript_id, task.as_str()) {
        return Ok(job);
    }
    if let Err(reason) = provider_ready(app, provider) {
        return Err(match reason {
            "model_missing" => AppError::ModelMissing(catalog::language_model(profile(app)).id.into()),
            "local_only" => AppError::LocalOnly,
            "no_consent" => AppError::ConsentRequired(provider.as_str().into()),
            "no_key" => AppError::MissingKey,
            other => AppError::Unsupported(other.into()),
        });
    }
    Ok(enqueue(app, transcript_id, JobKind::Generate { task, provider, detail }))
}

fn open_runner<R: Runtime>(app: &AppHandle<R>, job: &Job, provider: ProviderId) -> AppResult<(Box<dyn providers::Runner>, Option<models::InUse>)> {
    let p = prefs(app);
    match provider {
        ProviderId::Framely => {
            let spec = catalog::language_model(profile(app));
            let mgr = models(app);
            let path = mgr.ready_path(spec).ok_or_else(|| AppError::ModelMissing(spec.id.into()))?;
            let guard = mgr.use_model(spec);
            let hw = hardware(app);
            let worker = llm::Worker::start(&llm::worker_path()?, &path, spec.context, hw.accel == "metal", job.cancel.clone())?;
            Ok((Box::new(providers::FramelyRunner { worker, model: spec.id.into() }), Some(guard)))
        }
        #[cfg(target_os = "macos")]
        ProviderId::Apple => Ok((Box::new(providers::AppleRunner), None)),
        #[cfg(not(target_os = "macos"))]
        ProviderId::Apple => Err(AppError::Unsupported("apple".into())),
        ext => {
            let key = crate::platform::keystore::get(providers::key_account(ext))?.ok_or(AppError::MissingKey)?;
            if let Some(reason) = providers::external_reason(&p, ext, true) {
                return Err(match reason {
                    "local_only" => AppError::LocalOnly,
                    "no_consent" => AppError::ConsentRequired(ext.as_str().into()),
                    other => AppError::Unsupported(other.into()),
                });
            }
            let agent = providers::external_agent(p.local_only)?;
            Ok(if ext == ProviderId::Anthropic {
                (Box::new(providers::AnthropicRunner { agent, key, model: p.anthropic_model.clone() }), None)
            } else {
                (Box::new(providers::OpenAiRunner { agent, key, model: p.openai_model.clone() }), None)
            })
        }
    }
}

fn run_generation<R: Runtime>(app: &AppHandle<R>, job: &Job, task: Task, provider: ProviderId, detail: Detail) -> AppResult<()> {
    let t = with_db(app, |db| db.get(job.transcript_id))?;
    let segments = with_db(app, |db| db.segments(t.id))?;
    if segments.is_empty() {
        return Err(AppError::NoAudio);
    }
    emit_status(app, base_status(job, "running", "loading"));
    let started = Instant::now();
    let (mut runner, _guard) = open_runner(app, job, provider)?;
    let language = t.detected_language.clone().or_else(|| (t.language != "auto").then(|| t.language.clone()));
    let system = generation::system_prompt(language.as_deref());
    let lines = transcript::to_prompt_lines(&segments);
    let full_text: String = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    let max_out = generation::max_tokens(task, detail).min(runner.context_tokens() / 2);
    let mut budget = (runner.context_tokens() as usize)
        .saturating_sub(max_out as usize + generation::estimate_tokens(&system) + 400)
        .max(600);

    let content = loop {
        match generate_parts(app, job, runner.as_mut(), task, detail, &system, &lines, budget, max_out, &segments, t.media.duration_ms, &full_text) {
            // The estimate was optimistic for this model's tokenizer: smaller parts.
            Err(AppError::Provider(m)) if m == "context_overflow" && budget > 800 => budget = budget * 2 / 3,
            other => break other?,
        }
    };
    emit_status(app, JobStatus { progress: Some(1.0), ..base_status(job, "running", "saving") });
    let g = Generated {
        detail: generation::detail_name(detail).into(),
        provider: provider.as_str().into(),
        model: runner.model_name(),
        content,
        created_at: chrono::Utc::now().timestamp_millis(),
        edited: false,
    };
    with_db(app, |db| db.set_generated(t.id, task.as_str(), &g))?;
    if provider == ProviderId::Framely {
        let spec = catalog::language_model(profile(app));
        let _ = with_db(app, |db| db.record_perf(task.as_str(), spec.id, hardware(app).accel, full_text.chars().count() as f64, started.elapsed().as_millis() as u64));
    }
    let _ = app.emit(TRANSCRIPT_EVENT, t.id);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn generate_parts<R: Runtime>(
    app: &AppHandle<R>,
    job: &Job,
    runner: &mut dyn providers::Runner,
    task: Task,
    detail: Detail,
    system: &str,
    lines: &[(u64, String)],
    budget: usize,
    max_out: u32,
    segments: &[Segment],
    duration_ms: u64,
    full_text: &str,
) -> AppResult<Value> {
    let parts = generation::split_lines(lines, budget);
    let n = parts.len();
    let reduce_steps = usize::from(task == Task::Summary && n > 1);
    let total_steps = (n + reduce_steps) as f64;
    let meter = Meter::new(0.0);
    let mut results = Vec::with_capacity(n);
    for (i, part) in parts.iter().enumerate() {
        if job.cancel.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }
        let part_detail = if n > 1 && task == Task::Summary { Detail::Balanced } else { detail };
        let user = generation::user_prompt(task, part_detail, part, (n > 1).then_some((i, n)));
        let mut report = |f: f64| {
            let p = (i as f64 + f) / total_steps;
            emit_status(
                app,
                JobStatus { progress: Some(p), eta_ms: meter.eta(p), part: (n > 1).then_some((i + 1, n)), ..base_status(job, "running", "generating") },
            );
        };
        report(0.0);
        let raw = generation::parse_json(&runner.run(task, system, &user, max_out, &mut report)?)?;
        results.push(match task {
            Task::Summary => generation::normalize_summary(&raw, segments, duration_ms),
            Task::Topics => generation::normalize_topics(&raw, segments, duration_ms, full_text),
        });
    }
    if task == Task::Topics {
        return Ok(if n == 1 { results.remove(0) } else { generation::merge_topics(&results) });
    }
    // Summary: combine part summaries (in groups that fit, until one remains).
    let mut level = results;
    while level.len() > 1 {
        if job.cancel.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }
        emit_status(app, JobStatus { progress: Some(n as f64 / total_steps), ..base_status(job, "running", "combining") });
        let mut next = Vec::new();
        let mut group: Vec<Value> = Vec::new();
        let flush = |group: &mut Vec<Value>, runner: &mut dyn providers::Runner, next: &mut Vec<Value>| -> AppResult<()> {
            if group.len() == 1 {
                next.push(group.remove(0));
                return Ok(());
            }
            let prompt = generation::reduce_prompt(detail, group);
            let raw = generation::parse_json(&runner.run(task, system, &prompt, max_out, &mut |_| {})?)?;
            next.push(generation::normalize_summary(&raw, segments, duration_ms));
            group.clear();
            Ok(())
        };
        for r in level {
            let candidate = generation::estimate_tokens(&generation::reduce_prompt(detail, &[group.clone(), vec![r.clone()]].concat()));
            if !group.is_empty() && candidate > budget {
                flush(&mut group, runner, &mut next)?;
            }
            group.push(r);
        }
        if !group.is_empty() {
            flush(&mut group, runner, &mut next)?;
        }
        level = next;
    }
    Ok(level.remove(0))
}

// ---------------------------------------------------------------------------------------------
// Storage

/// Bytes used by models and by temporary audio.
pub fn storage<R: Runtime>(app: &AppHandle<R>) -> (u64, u64) {
    let temp = std::fs::read_dir(temp_dir(app))
        .map(|rd| rd.flatten().filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum())
        .unwrap_or(0);
    (models(app).disk_usage(), temp)
}

/// Deletes temporary audio not used by a running job (interrupted jobs re-decode on resume).
pub fn clear_temp<R: Runtime>(app: &AppHandle<R>) -> AppResult<u64> {
    let running = st(app).intelligence.current.lock().unwrap().as_ref().map(|j| j.transcript_id);
    let mut freed = 0;
    for e in std::fs::read_dir(temp_dir(app))?.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let id: Option<i64> = name.split('.').next().and_then(|s| s.parse().ok());
        if id.is_some() && id == running {
            continue;
        }
        freed += e.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = std::fs::remove_file(e.path());
    }
    Ok(freed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn wait<T>(what: &str, timeout_s: u64, mut f: impl FnMut() -> Option<T>) -> T {
        let start = Instant::now();
        loop {
            if let Some(v) = f() {
                return v;
            }
            assert!(start.elapsed().as_secs() < timeout_s, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    fn wait_job<R: Runtime>(app: &AppHandle<R>, id: u64) -> JobStatus {
        let mut last = String::new();
        wait("job", 1800, || {
            let s = jobs(app).into_iter().find(|j| j.id == id)?;
            let line = format!("{} {} {:?} eta={:?} part={:?}", s.state, s.phase, s.progress.map(|p| (p * 100.0).round()), s.eta_ms, s.part);
            if line != last {
                println!("  job {id}: {line}");
                last = line;
            }
            matches!(s.state, "done" | "failed" | "cancelled").then_some(s)
        })
    }

    /// The whole FrameLy AI pipeline against the real services and runtimes: model downloads
    /// (with SHA-256 checks) into FrameLy's data folder, native audio decoding, speech
    /// recognition, the framely-llm process for summaries and topics, Apple Intelligence,
    /// resumption and exports. Needs network, ~3.5 GB of disk and:
    ///   FRAMELY_TEST_MEDIA=<audio or video>  FRAMELY_LLM_WORKER=<framely-llm binary>
    #[test]
    #[ignore = "downloads models and runs them (minutes)"]
    fn end_to_end_with_real_models() {
        let media = PathBuf::from(std::env::var("FRAMELY_TEST_MEDIA").expect("FRAMELY_TEST_MEDIA"));
        let mut ctx = tauri::test::mock_context(tauri::test::noop_assets());
        ctx.config_mut().identifier = "app.framely.desktop".into();
        let app = tauri::test::mock_builder().build(ctx).expect("mock app");
        let handle = app.handle().clone();
        let tmp = tempfile::tempdir().unwrap();
        let dirs = crate::core::settings::DefaultDirs { videos: tmp.path().join("v"), pictures: tmp.path().join("p") };
        let settings = crate::core::settings::SettingsStore::load(tmp.path().join("settings.json"), &dirs).unwrap();
        handle.manage(AppState {
            settings,
            backend: Arc::from(crate::platform::backend()),
            screenshots: Default::default(),
            recordings: Default::default(),
            library: Default::default(),
            intelligence: Default::default(),
            log_dir: tmp.path().to_path_buf(),
            start_hidden: AtomicBool::new(true),
            _log_guard: None,
        });
        init(&handle).unwrap();
        let hw = hardware(&handle);
        println!("hardware: {hw:?}\nprofile: {:?}", profile(&handle));

        // 1. Models: real downloads, verified.
        let mgr = models(&handle);
        let speech = catalog::speech_model(catalog::precision_for(profile(&handle), crate::core::settings::Precision::Auto));
        let language = catalog::language_model(profile(&handle));
        for spec in [speech, language] {
            if mgr.ready_path(spec).is_none() {
                mgr.download(spec, false).unwrap();
                let started = Instant::now();
                let s = wait("download", 3600, || {
                    let s = mgr.status_of(spec);
                    (!matches!(s.state, models::State::Downloading | models::State::Verifying)).then_some(s)
                });
                println!("{}: {:?} in {:?} ({:?})", spec.id, s.state, started.elapsed(), s.error);
            }
            assert!(mgr.ready_path(spec).is_some(), "{} installed", spec.id);
        }

        // 2. Transcription.
        let started = Instant::now();
        let (t, job) = start_transcription(&handle, &media, None, None).unwrap();
        if let Some(job) = job {
            let s = wait_job(&handle, job);
            assert_eq!(s.state, "done", "{:?}", s.error);
        }
        let t = with_db(&handle, |db| db.get(t.id)).unwrap();
        let segments = with_db(&handle, |db| db.segments(t.id)).unwrap();
        println!("transcribed {} ms of audio in {:?}: {} segments, language {:?}", t.media.duration_ms, started.elapsed(), segments.len(), t.detected_language);
        assert_eq!(t.status, Status::Done);
        assert!(!segments.is_empty());
        println!("{}", transcript::to_txt(&segments, true));

        // Same file + settings again: reused, no new job.
        let (again, job) = start_transcription(&handle, &media, None, None).unwrap();
        assert_eq!((again.id, job), (t.id, None));

        // 3. Summary and topics with FrameLy AI (framely-llm process).
        for task in [Task::Summary, Task::Topics] {
            let started = Instant::now();
            let job = start_generation(&handle, t.id, task, ProviderId::Framely, Detail::Balanced).unwrap();
            let s = wait_job(&handle, job);
            assert_eq!(s.state, "done", "{:?}", s.error);
            let g = with_db(&handle, |db| db.generated(t.id, task.as_str())).unwrap().unwrap();
            println!("{} ({:?}): {}", task.as_str(), started.elapsed(), serde_json::to_string_pretty(&g.content).unwrap());
        }

        // 4. Apple Intelligence, when this Mac has it.
        #[cfg(target_os = "macos")]
        if provider_ready(&handle, ProviderId::Apple).is_ok() {
            for task in [Task::Summary, Task::Topics] {
                let job = start_generation(&handle, t.id, task, ProviderId::Apple, Detail::Brief).unwrap();
                let s = wait_job(&handle, job);
                assert_eq!(s.state, "done", "{:?}", s.error);
                let g = with_db(&handle, |db| db.generated(t.id, task.as_str())).unwrap().unwrap();
                println!("apple {}: {}", task.as_str(), serde_json::to_string_pretty(&g.content).unwrap());
            }
            // Leave FrameLy AI's results in place for the app.
            for task in [Task::Summary, Task::Topics] {
                let job = start_generation(&handle, t.id, task, ProviderId::Framely, Detail::Balanced).unwrap();
                assert_eq!(wait_job(&handle, job).state, "done");
            }
        }

        // 5. External providers stay blocked without switch, key and consent.
        assert!(start_generation(&handle, t.id, Task::Summary, ProviderId::Anthropic, Detail::Brief).is_err());

        // 6. Exports.
        let out = tmp.path().join("export.pdf");
        let summary = with_db(&handle, |db| db.generated(t.id, "summary")).unwrap().unwrap();
        let md = generation::summary_markdown(&summary.content);
        assert!(!md.trim().is_empty());
        let blocks: Vec<crate::core::pdf::Block> = transcript::paragraphs(&segments)
            .iter()
            .map(|p| crate::core::pdf::Block::Para { time: Some(transcript::fmt_clock(p.start_ms)), text: p.segments.iter().map(|&i| segments[i as usize].text.clone()).collect::<Vec<_>>().join(" ") })
            .collect();
        std::fs::write(&out, crate::core::pdf::render(&blocks, "test")).unwrap();
        assert!(std::fs::metadata(&out).unwrap().len() > 1000);
        assert!(transcript::to_srt(&segments).contains(" --> "));
    }
}
