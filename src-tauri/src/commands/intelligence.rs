//! IPC for transcription, summaries, topics, models and providers (FrameLy AI).

use crate::{
    core::{
        errors::{AppError, AppResult},
        pdf,
        settings::{Detail, ProviderId},
        transcript::{self, Paragraph, Segment},
        transcripts::{Generated, MediaRef, Transcript},
    },
    intelligence::{
        self as ai, catalog, estimate,
        generation::{self, Labels, Task},
        hardware::Hardware,
        models::ModelStatus,
        providers::{self, ProviderInfo},
        JobStatus,
    },
    AppState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| AppError::Runtime(e.to_string()))?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    pub models_bytes: u64,
    pub temp_bytes: u64,
    pub free_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub hardware: Hardware,
    /// Profile in effect (the user's choice or the recommendation).
    pub profile: crate::core::settings::AiProfile,
    /// Precision used when "automatic".
    pub default_precision: &'static str,
    pub models: Vec<ModelStatus>,
    pub providers: Vec<ProviderInfo>,
    pub storage: Storage,
    pub importable: &'static [&'static str],
    pub anthropic_models: &'static [&'static str],
}

fn provider_infos(app: &AppHandle) -> Vec<ProviderInfo> {
    let p = ai::prefs(app);
    let profile = ai::profile(app);
    let lm = catalog::language_model(profile);
    let mut out = vec![ProviderInfo {
        id: ProviderId::Framely,
        local: true,
        needs_internet: false,
        needs_key: false,
        key_configured: false,
        available: ai::provider_ready(app, ProviderId::Framely).is_ok(),
        reason: ai::provider_ready(app, ProviderId::Framely).err().map(str::to_string),
        context_tokens: lm.context,
        paid: false,
        model: lm.id.into(),
        consented: true,
    }];
    if cfg!(target_os = "macos") {
        let r = ai::provider_ready(app, ProviderId::Apple);
        out.push(ProviderInfo {
            id: ProviderId::Apple,
            local: true,
            needs_internet: false,
            needs_key: false,
            key_configured: false,
            available: r.is_ok(),
            reason: r.err().map(str::to_string),
            context_tokens: 4_096,
            paid: false,
            model: "apple-foundation-model".into(),
            consented: true,
        });
    }
    for (id, model) in [(ProviderId::Anthropic, p.anthropic_model.clone()), (ProviderId::Openai, p.openai_model.clone())] {
        let key = crate::platform::keystore::get(providers::key_account(id)).ok().flatten().is_some();
        let reason = providers::external_reason(&p, id, key);
        out.push(ProviderInfo {
            id,
            local: false,
            needs_internet: true,
            needs_key: true,
            key_configured: key,
            available: reason.is_none(),
            reason: reason.map(str::to_string),
            context_tokens: if id == ProviderId::Anthropic { 200_000 } else { 100_000 },
            paid: true,
            model,
            consented: p.consents.contains(&id),
        });
    }
    out
}

fn storage(app: &AppHandle) -> Storage {
    let (models_bytes, temp_bytes) = ai::storage(app);
    Storage { models_bytes, temp_bytes, free_bytes: crate::platform::disk::free_bytes(ai::models(app).dir().parent().unwrap_or(std::path::Path::new("/"))) }
}

#[tauri::command]
pub async fn ai_overview(app: AppHandle) -> AppResult<Overview> {
    blocking(move || {
        let profile = ai::profile(&app);
        Ok(Overview {
            hardware: ai::hardware(&app),
            profile,
            default_precision: catalog::precision_name(catalog::precision_for(profile, crate::core::settings::Precision::Auto)),
            models: ai::models(&app).all(),
            providers: provider_infos(&app),
            storage: storage(&app),
            importable: ai::importable_extensions(),
            anthropic_models: providers::ANTHROPIC_MODELS,
        })
    })
    .await
}

#[tauri::command]
pub async fn ai_model_action(app: AppHandle, id: String, action: String) -> AppResult<ModelStatus> {
    blocking(move || {
        let spec = catalog::spec(&id).ok_or(AppError::NotFound)?;
        let mgr = ai::models(&app);
        let local_only = ai::prefs(&app).local_only;
        match action.as_str() {
            "download" => mgr.download(spec, local_only)?,
            "pause" => mgr.pause(spec),
            "cancel" => mgr.cancel(spec),
            "remove" => mgr.remove(spec)?,
            "verify" => {
                mgr.verify(spec)?;
            }
            "repair" => mgr.repair(spec, local_only)?,
            _ => return Err(AppError::NotFound),
        }
        Ok(mgr.status_of(spec))
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    pub media: MediaRef,
    pub precision: &'static str,
    pub model: ModelStatus,
    pub time: estimate::TimeEstimate,
    pub accel: &'static str,
    /// Existing transcript of this file, if any (finished ones are reused).
    pub existing: Option<Transcript>,
    /// Before the transcript exists: what generating a summary/topics locally takes, and
    /// what sending it out would roughly cost (tokens, and US$ for providers with known prices).
    pub summary_local: estimate::TimeEstimate,
    pub topics_local: estimate::TimeEstimate,
    pub input_tokens: usize,
    pub anthropic_cost_usd: Option<(f64, f64)>,
}

#[tauri::command]
pub async fn ai_estimate(app: AppHandle, path: PathBuf, precision: Option<String>) -> AppResult<Estimate> {
    blocking(move || {
        let media = ai::probe(&app, &path)?;
        let precision = catalog::precision_for(ai::profile(&app), catalog::parse_precision(precision.as_deref().unwrap_or("auto")));
        let spec = catalog::speech_model(precision);
        let hw = ai::hardware(&app);
        let history = ai::with_db(&app, |db| db.perf_rate("asr", spec.id, hw.accel))?;
        let existing = ai::with_db(&app, |db| db.find_by_path(&media.path))?;
        let chars = estimate::expected_chars(media.duration_ms);
        let lm = catalog::language_model(ai::profile(&app));
        let local = |task: &str| {
            let history = ai::with_db(&app, |db| db.perf_rate(task, lm.id, hw.accel)).ok().flatten();
            estimate::generation(chars, task, lm.id, hw.accel, history)
        };
        let input_tokens = chars / 3 + 600;
        Ok(Estimate {
            summary_local: local("summary"),
            topics_local: local("topics"),
            input_tokens,
            anthropic_cost_usd: anthropic_cost(&ai::prefs(&app).anthropic_model, input_tokens),
            time: estimate::transcription(media.duration_ms, spec.id, hw.accel, history),
            precision: catalog::precision_name(precision),
            model: ai::models(&app).status_of(spec),
            accel: hw.accel,
            media,
            existing,
        })
    })
    .await
}

/// Input plus 1k–12k output tokens (the model's reasoning included) at the listed prices.
fn anthropic_cost(model: &str, input_tokens: usize) -> Option<(f64, f64)> {
    providers::anthropic_price(model).map(|(inp, out)| {
        let i = input_tokens as f64 / 1e6 * inp;
        (i + 1_000.0 / 1e6 * out, i + 12_000.0 / 1e6 * out)
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationEstimate {
    pub time: Option<estimate::TimeEstimate>,
    /// Approximate input tokens sent (external providers).
    pub input_tokens: usize,
    /// Approximate cost in USD for providers with known prices (min, max).
    pub cost_usd: Option<(f64, f64)>,
}

#[tauri::command]
pub async fn ai_generation_estimate(app: AppHandle, id: i64, task: String, provider: ProviderId) -> AppResult<GenerationEstimate> {
    blocking(move || {
        let segments = ai::with_db(&app, |db| db.segments(id))?;
        let text: String = transcript::to_prompt_lines(&segments).into_iter().map(|(_, l)| l + "\n").collect();
        let chars = text.chars().count();
        let input_tokens = generation::estimate_tokens(&text) + 600;
        let task_name = if task == "topics" { "topics" } else { "summary" };
        let time = (provider == ProviderId::Framely).then(|| {
            let spec = catalog::language_model(ai::profile(&app));
            let hw = ai::hardware(&app);
            let history = ai::with_db(&app, |db| db.perf_rate(task_name, spec.id, hw.accel)).ok().flatten();
            estimate::generation(chars, task_name, spec.id, hw.accel, history)
        });
        let cost_usd = (provider == ProviderId::Anthropic).then(|| anthropic_cost(&ai::prefs(&app).anthropic_model, input_tokens)).flatten();
        Ok(GenerationEstimate { time, input_tokens, cost_usd })
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Started {
    pub transcript: Transcript,
    pub job_id: Option<u64>,
}

#[tauri::command]
pub async fn transcription_start(
    app: AppHandle,
    path: PathBuf,
    precision: Option<String>,
    language: Option<String>,
    then: Option<ai::Followups>,
) -> AppResult<Started> {
    blocking(move || {
        let (transcript, job_id) = ai::start_transcription(&app, &path, precision, language, then)?;
        Ok(Started { transcript, job_id })
    })
    .await
}

#[tauri::command]
pub async fn transcripts_list(app: AppHandle) -> AppResult<Vec<Transcript>> {
    blocking(move || ai::with_db(&app, |db| db.list())).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detailed {
    pub transcript: Transcript,
    pub segments: Vec<Segment>,
    pub paragraphs: Vec<Paragraph>,
    pub summary: Option<Generated>,
    pub topics: Option<Generated>,
    /// The media file is still where it was transcribed from.
    pub media_available: bool,
}

#[tauri::command]
pub async fn transcript_get(app: AppHandle, id: i64) -> AppResult<Detailed> {
    blocking(move || {
        let (transcript, segments, summary, topics) =
            ai::with_db(&app, |db| Ok((db.get(id)?, db.segments(id)?, db.generated(id, "summary")?, db.generated(id, "topics")?)))?;
        let media_available = std::path::Path::new(&transcript.media.path).exists();
        if media_available {
            let _ = app.asset_protocol_scope().allow_file(&transcript.media.path);
        }
        Ok(Detailed { paragraphs: transcript::paragraphs(&segments), transcript, segments, summary, topics, media_available })
    })
    .await
}

#[tauri::command]
pub async fn transcript_find(app: AppHandle, path: String) -> AppResult<Option<Transcript>> {
    blocking(move || ai::with_db(&app, |db| db.find_by_path(&path))).await
}

#[tauri::command]
pub async fn transcript_edit(app: AppHandle, id: i64, idx: u32, text: String) -> AppResult<()> {
    blocking(move || ai::with_db(&app, |db| db.edit_segment(id, idx, &text))).await
}

#[tauri::command]
pub async fn transcript_delete(app: AppHandle, id: i64) -> AppResult<()> {
    blocking(move || ai::with_db(&app, |db| db.delete(id))).await
}

/// Manual edits of a summary or topics.
#[tauri::command]
pub async fn generated_save(app: AppHandle, id: i64, kind: String, content: Value) -> AppResult<()> {
    blocking(move || {
        if kind != "summary" && kind != "topics" {
            return Err(AppError::NotFound);
        }
        ai::with_db(&app, |db| {
            let mut g = db.generated(id, &kind)?.ok_or(AppError::NotFound)?;
            g.content = content;
            g.edited = true;
            db.set_generated(id, &kind, &g)
        })
    })
    .await
}

#[tauri::command]
pub async fn generation_start(app: AppHandle, id: i64, task: String, provider: ProviderId, detail: Detail) -> AppResult<u64> {
    blocking(move || {
        let task = if task == "topics" { Task::Topics } else { Task::Summary };
        ai::start_generation(&app, id, task, provider, detail)
    })
    .await
}

#[tauri::command]
pub fn ai_job_cancel(app: AppHandle, job_id: u64) {
    ai::cancel(&app, job_id);
}

#[tauri::command]
pub fn ai_jobs(app: AppHandle) -> Vec<JobStatus> {
    ai::jobs(&app)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRequest {
    pub id: i64,
    /// "transcript" | "summary" | "topics"
    pub what: String,
    /// "txt" | "srt" | "vtt" | "md" | "pdf"
    pub format: String,
    pub path: PathBuf,
    pub title: String,
    /// Line under the title (date, duration) for PDF/Markdown.
    pub meta: String,
    pub labels: Labels,
}

fn strip_markdown(md: &str) -> String {
    md.lines().map(|l| l.trim_start_matches("## ").replace("**", "")).collect::<Vec<_>>().join("\n")
}

fn markdown_to_blocks(md: &str) -> Vec<pdf::Block> {
    let mut out = Vec::new();
    for line in md.lines() {
        let trimmed = line.trim_start();
        let level = ((line.len() - trimmed.len()) / 2) as u8;
        if let Some(h) = trimmed.strip_prefix("## ") {
            out.push(pdf::Block::Heading(h.to_string()));
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            let item = item.replace("**", "");
            let (time, text) = match item.strip_prefix('[').and_then(|r| r.split_once("] ")) {
                Some((t, rest)) if transcript::parse_clock(t).is_some() => (Some(t.to_string()), rest.to_string()),
                _ => (None, item),
            };
            out.push(pdf::Block::Bullet { level, time, text });
        } else if !trimmed.is_empty() {
            out.push(pdf::Block::Para { time: None, text: trimmed.to_string() });
        }
    }
    out
}

#[tauri::command]
pub async fn transcript_export(app: AppHandle, request: ExportRequest) -> AppResult<()> {
    blocking(move || {
        let r = request;
        if !r.path.is_absolute() {
            return Err(AppError::InvalidName(r.path.display().to_string()));
        }
        let (segments, summary, topics) =
            ai::with_db(&app, |db| Ok((db.segments(r.id)?, db.generated(r.id, "summary")?, db.generated(r.id, "topics")?)))?;
        let md_body = match r.what.as_str() {
            "transcript" => None,
            "summary" => Some(generation::summary_markdown(&summary.ok_or(AppError::NotFound)?.content)),
            "topics" => Some(generation::topics_markdown(&topics.ok_or(AppError::NotFound)?.content, &r.labels)),
            _ => return Err(AppError::NotFound),
        };
        let bytes: Vec<u8> = match (r.format.as_str(), &md_body) {
            ("srt", None) => transcript::to_srt(&segments).into_bytes(),
            ("vtt", None) => transcript::to_vtt(&segments).into_bytes(),
            ("txt", None) => transcript::to_txt(&segments, true).into_bytes(),
            ("md", None) => transcript::to_markdown(&r.title, &segments).replacen('\n', &format!("\n\n_{}_\n", r.meta), 1).into_bytes(),
            ("txt", Some(md)) => format!("{}\n{}\n\n{}", r.title, r.meta, strip_markdown(md)).into_bytes(),
            ("md", Some(md)) => format!("# {}\n\n_{}_\n\n{md}", r.title, r.meta).into_bytes(),
            ("pdf", body) => {
                let mut blocks = vec![pdf::Block::Title(r.title.clone()), pdf::Block::Meta(r.meta.clone())];
                match body {
                    None => blocks.extend(transcript::paragraphs(&segments).iter().map(|p| pdf::Block::Para {
                        time: Some(transcript::fmt_clock(p.start_ms)),
                        text: p.segments.iter().map(|&i| segments[i as usize].text.trim()).collect::<Vec<_>>().join(" "),
                    })),
                    Some(md) => blocks.extend(markdown_to_blocks(md)),
                }
                let all: String = blocks
                    .iter()
                    .map(|b| match b {
                        pdf::Block::Title(t) | pdf::Block::Meta(t) | pdf::Block::Heading(t) => t.clone(),
                        pdf::Block::Para { text, .. } | pdf::Block::Bullet { text, .. } => text.clone(),
                    })
                    .collect();
                if !pdf::supports(&all) {
                    return Err(AppError::Unsupported("pdf_text".into()));
                }
                pdf::render(&blocks, &r.title)
            }
            _ => return Err(AppError::Unsupported(r.format.clone())),
        };
        let tmp = r.path.with_extension("framely-tmp");
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &r.path)?;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn provider_key_set(app: AppHandle, provider: ProviderId, key: String) -> AppResult<()> {
    blocking(move || {
        if !provider.is_external() {
            return Err(AppError::NotFound);
        }
        let key = key.trim();
        if key.len() < 8 || key.chars().any(char::is_whitespace) {
            return Err(AppError::InvalidSettings("api key".into()));
        }
        crate::platform::keystore::set(providers::key_account(provider), key)?;
        let _ = app;
        Ok(())
    })
    .await
}

/// Removes the key, the consent and the provider assignments that used it.
#[tauri::command]
pub async fn provider_remove(app: AppHandle, provider: ProviderId) -> AppResult<crate::core::settings::Settings> {
    blocking(move || {
        if !provider.is_external() {
            return Err(AppError::NotFound);
        }
        crate::platform::keystore::delete(providers::key_account(provider))?;
        let state = app.state::<AppState>();
        let mut s = state.settings.get();
        let i = &mut s.intelligence;
        i.consents.retain(|c| *c != provider);
        if i.summary_provider == provider {
            i.summary_provider = ProviderId::Framely;
        }
        if i.topics_provider == provider {
            i.topics_provider = ProviderId::Framely;
        }
        if provider == ProviderId::Openai {
            i.openai_model.clear();
        }
        state.settings.replace(s)
    })
    .await
}

/// Checks the key against the provider (no transcript content is sent) and lists its models.
#[tauri::command]
pub async fn provider_test(app: AppHandle, provider: ProviderId) -> AppResult<Vec<String>> {
    blocking(move || {
        let agent = providers::external_agent(ai::prefs(&app).local_only)?;
        let key = crate::platform::keystore::get(providers::key_account(provider))?.ok_or(AppError::MissingKey)?;
        match provider {
            ProviderId::Anthropic => providers::AnthropicRunner::list_models(&agent, &key),
            ProviderId::Openai => providers::OpenAiRunner::list_models(&agent, &key),
            _ => Err(AppError::NotFound),
        }
    })
    .await
}

#[tauri::command]
pub async fn ai_consent(app: AppHandle, provider: ProviderId, granted: bool) -> AppResult<crate::core::settings::Settings> {
    blocking(move || {
        let state = app.state::<AppState>();
        let mut s = state.settings.get();
        s.intelligence.consents.retain(|c| *c != provider);
        if granted && provider.is_external() {
            s.intelligence.consents.push(provider);
        }
        state.settings.replace(s)
    })
    .await
}

#[tauri::command]
pub async fn ai_clear_temp(app: AppHandle) -> AppResult<u64> {
    blocking(move || ai::clear_temp(&app)).await
}

#[tauri::command]
pub async fn media_probe(app: AppHandle, path: PathBuf) -> AppResult<MediaRef> {
    blocking(move || ai::probe(&app, &path)).await
}
