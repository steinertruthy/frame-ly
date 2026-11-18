//! AI provider layer for summaries and topics. Each provider declares what it is (local or
//! external, internet, key, cost, context window, availability) and runs a prompt into JSON
//! that matches `generation::schema`. Callers never touch SDKs, HTTP or runtimes directly.
//!
//! - FrameLy AI Cloud: FrameLy's own service (`cloud.rs`; the default when the build has a
//!   cloud URL, see `core::settings::default_generation_provider`), consent required.
//! - FrameLy AI Local: local model in the framely-llm process (no account, no key, offline).
//! - Apple Intelligence: on-device Foundation Models (macOS 26+ only).
//! - Anthropic and OpenAI: optional, external, need an API key and explicit consent.

use super::{generation::Task, llm, net};
use crate::core::{
    errors::{AppError, AppResult},
    settings::{Intelligence, ProviderId},
};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: ProviderId,
    /// Runs on this computer.
    pub local: bool,
    pub needs_internet: bool,
    pub needs_key: bool,
    pub key_configured: bool,
    /// Can run right now.
    pub available: bool,
    /// Why not: "model_missing" | "local_only" | "external_disabled" | "no_key" | "no_consent" |
    /// "os_version" | "device_not_eligible" | "not_enabled" | "model_not_ready" | "unsupported_cpu".
    pub reason: Option<String>,
    pub context_tokens: u32,
    /// Usage is billed by the provider.
    pub paid: bool,
    pub model: String,
    pub consented: bool,
}

pub const ANTHROPIC_MODELS: &[&str] = &["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"];

/// Price per million tokens (input, output) for the Anthropic models offered, in USD, as listed
/// by Anthropic (Nov 2026). Shown as an estimate; the provider's bill is what counts.
pub fn anthropic_price(model: &str) -> Option<(f64, f64)> {
    match model {
        "claude-opus-5-5" => Some((4.0, 20.0)),
        "claude-sonnet-5-5" => Some((2.0, 10.0)),
        "claude-haiku-5-5" => Some((0.10, 0.50)),
        _ => None,
    }
}

pub fn key_account(id: ProviderId) -> &'static str {
    match id {
        ProviderId::Anthropic => "anthropic-api-key",
        _ => "openai-api-key",
    }
}

/// Availability of the external ones given the settings (pure; tested).
pub fn external_reason(s: &Intelligence, id: ProviderId, has_key: bool) -> Option<&'static str> {
    if s.local_only {
        Some("local_only")
    } else if !s.external_enabled {
        Some("external_disabled")
    } else if !has_key {
        Some("no_key")
    } else if id == ProviderId::Openai && s.openai_model.is_empty() {
        Some("no_model")
    } else if !s.consents.contains(&id) {
        Some("no_consent")
    } else {
        None
    }
}

/// What to generate, before it becomes a prompt.
pub enum Input<'a> {
    /// Transcript lines ("[mm:ss] text"), optionally one part of a longer transcript.
    Part { text: &'a str, part: Option<(usize, usize)> },
    /// Partial summaries (normalized) of consecutive parts, to combine.
    Combine { partials: &'a [Value] },
}

pub struct Request<'a> {
    pub task: Task,
    pub detail: crate::core::settings::Detail,
    pub language: Option<&'a str>,
    pub system: &'a str,
    pub input: Input<'a>,
}

/// Availability of FrameLy Cloud given the settings (pure; tested). No key, no model and not
/// behind the "external providers" switch — but the text leaves the computer, so local-only
/// mode blocks it and the first use asks for consent.
pub fn cloud_reason(s: &Intelligence, configured: bool) -> Option<&'static str> {
    if s.local_only {
        Some("local_only")
    } else if !configured {
        Some("cloud_not_configured")
    } else if !s.consents.contains(&ProviderId::Cloud) {
        Some("no_consent")
    } else {
        None
    }
}

/// Something that turns a request into JSON (`generation::schema`).
pub trait Runner {
    /// Context window (tokens) available for prompt + answer.
    fn context_tokens(&self) -> u32;
    fn model_name(&self) -> String;
    fn run(
        &mut self,
        task: Task,
        system: &str,
        user: &str,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String>;

    /// Builds the prompt and runs it. Services that build their own prompts (FrameLy Cloud)
    /// override this and send the structured request instead.
    fn generate(
        &mut self,
        req: &Request<'_>,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String> {
        let user = match req.input {
            Input::Part { text, part } => super::generation::user_prompt(req.task, req.detail, text, part),
            Input::Combine { partials } => super::generation::reduce_prompt(req.detail, partials),
        };
        self.run(req.task, req.system, &user, max_tokens, progress)
    }
}

pub struct FramelyRunner {
    pub worker: llm::Worker,
    pub model: String,
}

impl Runner for FramelyRunner {
    fn context_tokens(&self) -> u32 {
        self.worker.ctx
    }
    fn model_name(&self) -> String {
        self.model.clone()
    }
    fn run(
        &mut self,
        task: Task,
        system: &str,
        user: &str,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String> {
        let schema = super::generation::schema(task);
        // Prompt evaluation is measurable; generation length isn't known in advance, so it
        // advances towards the token limit (and jumps to done when the model stops).
        let out = self.worker.generate(system, user, &schema, max_tokens, &mut |phase, done, total| {
            let f = if total == 0 { 0.0 } else { done as f64 / total as f64 };
            progress(if phase == "prompt" { f * 0.3 } else { 0.3 + f * 0.7 });
        })?;
        if out.truncated {
            return Err(AppError::Provider("response too long".into()));
        }
        Ok(out.text)
    }
}

#[cfg(target_os = "macos")]
pub struct AppleRunner;

#[cfg(target_os = "macos")]
impl Runner for AppleRunner {
    fn context_tokens(&self) -> u32 {
        4_096
    }
    fn model_name(&self) -> String {
        "apple-foundation-model".into()
    }
    fn run(
        &mut self,
        task: Task,
        system: &str,
        user: &str,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String> {
        progress(0.0);
        let r = crate::platform::apple_ai::generate(&json!({
            "task": task.as_str(), "instructions": system, "prompt": user, "maxTokens": max_tokens
        }));
        progress(1.0);
        r
    }
}

fn http_error(provider: &str, status: u16, body: &str) -> AppError {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_string))
        .unwrap_or_default();
    match status {
        401 | 403 => AppError::Provider(format!("{provider}: invalid API key")),
        429 => AppError::Provider(format!("{provider}: rate limit or quota reached")),
        _ => AppError::Provider(format!("{provider}: HTTP {status} {detail}")),
    }
}

pub struct AnthropicRunner {
    pub agent: ureq::Agent,
    pub key: String,
    pub model: String,
}

impl AnthropicRunner {
    fn request(&self, body: &Value) -> AppResult<Value> {
        let mut req = self
            .agent
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.key)
            .header("anthropic-version", "2026-10-21")
            .header("content-type", "application/json");
        if body.get("fallbacks").is_some() {
            req = req.header("anthropic-beta", "server-side-fallback-2026-07-01");
        }
        let mut resp = req.send(body.to_string()).map_err(net::net_err)?;
        let status = resp.status().as_u16();
        let text = resp.body_mut().read_to_string().map_err(net::net_err)?;
        if status != 200 {
            return Err(http_error("Anthropic", status, &text));
        }
        Ok(serde_json::from_str(&text)?)
    }

    /// Model ids the key can use (connection test).
    pub fn list_models(agent: &ureq::Agent, key: &str) -> AppResult<Vec<String>> {
        let mut resp = agent
            .get("https://api.anthropic.com/v1/models?limit=100")
            .header("x-api-key", key)
            .header("anthropic-version", "2026-10-21")
            .call()
            .map_err(net::net_err)?;
        let status = resp.status().as_u16();
        let text = resp.body_mut().read_to_string().map_err(net::net_err)?;
        if status != 200 {
            return Err(http_error("Anthropic", status, &text));
        }
        let v: Value = serde_json::from_str(&text)?;
        Ok(v["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m["id"].as_str().map(str::to_string))
            .collect())
    }
}

impl Runner for AnthropicRunner {
    fn context_tokens(&self) -> u32 {
        // The models offered accept 1M tokens; FrameLy sends at most this much per request to
        // keep each call (and its cost) bounded.
        200_000
    }
    fn model_name(&self) -> String {
        self.model.clone()
    }
    fn run(
        &mut self,
        task: Task,
        system: &str,
        user: &str,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String> {
        progress(0.0);
        let mut body = json!({
            "model": self.model,
            "max_tokens": max_tokens.max(4_000) * 2,
            "system": system,
            "messages": [{"role": "user", "content": user}],
            "output_config": {"effort": "medium", "format": {"type": "json_schema", "schema": super::generation::schema(task)}},
        });
        // Refusal fallback (server-side) on the models that support it.
        if self.model == "claude-opus-5-5" || self.model == "claude-sonnet-5-5" {
            body["fallbacks"] = json!("default");
        }
        let v = self.request(&body)?;
        progress(1.0);
        if v["stop_reason"] == "refusal" {
            return Err(AppError::Provider("Anthropic declined this content".into()));
        }
        if v["stop_reason"] == "max_tokens" {
            return Err(AppError::Provider("response too long".into()));
        }
        v["content"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|b| b["type"] == "text")
            .and_then(|b| b["text"].as_str())
            .map(str::to_string)
            .ok_or_else(|| AppError::Provider("Anthropic: empty response".into()))
    }
}

pub struct OpenAiRunner {
    pub agent: ureq::Agent,
    pub key: String,
    pub model: String,
}

impl OpenAiRunner {
    pub fn list_models(agent: &ureq::Agent, key: &str) -> AppResult<Vec<String>> {
        let mut resp = agent
            .get("https://api.openai.com/v1/models")
            .header("authorization", &format!("Bearer {key}"))
            .call()
            .map_err(net::net_err)?;
        let status = resp.status().as_u16();
        let text = resp.body_mut().read_to_string().map_err(net::net_err)?;
        if status != 200 {
            return Err(http_error("OpenAI", status, &text));
        }
        let v: Value = serde_json::from_str(&text)?;
        let mut ids: Vec<String> = v["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m["id"].as_str().map(str::to_string))
            .collect();
        // Chat models only (no embeddings, audio, images or moderation).
        ids.retain(|id| {
            (id.starts_with("gpt-") || id.starts_with('o'))
                && ![
                    "embedding",
                    "audio",
                    "realtime",
                    "tts",
                    "transcribe",
                    "image",
                    "search",
                    "moderation",
                    "dall",
                ]
                .iter()
                .any(|x| id.contains(x))
        });
        ids.sort();
        Ok(ids)
    }
}

impl Runner for OpenAiRunner {
    fn context_tokens(&self) -> u32 {
        100_000
    }
    fn model_name(&self) -> String {
        self.model.clone()
    }
    fn run(
        &mut self,
        task: Task,
        system: &str,
        user: &str,
        max_tokens: u32,
        progress: &mut dyn FnMut(f64),
    ) -> AppResult<String> {
        progress(0.0);
        let body = json!({
            "model": self.model,
            "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
            "response_format": {"type": "json_schema", "json_schema": {"name": format!("framely_{}", task.as_str()), "strict": true, "schema": super::generation::schema(task)}},
            "max_completion_tokens": max_tokens.max(4_000) * 2,
        });
        let mut resp = self
            .agent
            .post("https://api.openai.com/v1/chat/completions")
            .header("authorization", &format!("Bearer {}", self.key))
            .header("content-type", "application/json")
            .send(body.to_string())
            .map_err(net::net_err)?;
        let status = resp.status().as_u16();
        let text = resp.body_mut().read_to_string().map_err(net::net_err)?;
        if status != 200 {
            return Err(http_error("OpenAI", status, &text));
        }
        progress(1.0);
        let v: Value = serde_json::from_str(&text)?;
        let msg = &v["choices"][0]["message"];
        if let Some(r) = msg["refusal"].as_str() {
            return Err(AppError::Provider(format!("OpenAI declined: {r}")));
        }
        if v["choices"][0]["finish_reason"] == "length" {
            return Err(AppError::Provider("response too long".into()));
        }
        msg["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| AppError::Provider("OpenAI: empty response".into()))
    }
}

/// HTTP client for an external provider (refuses in local-only mode).
pub fn external_agent(local_only: bool) -> AppResult<ureq::Agent> {
    net::agent(local_only, Some(Duration::from_secs(600)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_providers_need_switch_key_and_consent() {
        let mut s = Intelligence::default();
        assert_eq!(external_reason(&s, ProviderId::Anthropic, true), Some("external_disabled"));
        s.external_enabled = true;
        assert_eq!(external_reason(&s, ProviderId::Anthropic, false), Some("no_key"));
        assert_eq!(external_reason(&s, ProviderId::Anthropic, true), Some("no_consent"));
        assert_eq!(external_reason(&s, ProviderId::Openai, true), Some("no_model"));
        s.consents.push(ProviderId::Anthropic);
        assert_eq!(external_reason(&s, ProviderId::Anthropic, true), None);
        s.local_only = true;
        assert_eq!(
            external_reason(&s, ProviderId::Anthropic, true),
            Some("local_only"),
            "local-only wins over everything"
        );
        assert!(anthropic_price(ANTHROPIC_MODELS[0]).is_some());
    }

    #[test]
    fn cloud_needs_no_key_or_switch_but_consent_and_network() {
        let mut s = Intelligence::default();
        assert!(!s.external_enabled);
        assert_eq!(cloud_reason(&s, false), Some("cloud_not_configured"));
        assert_eq!(cloud_reason(&s, true), Some("no_consent"));
        s.consents.push(ProviderId::Cloud);
        assert_eq!(cloud_reason(&s, true), None, "works without the external-providers switch");
        s.local_only = true;
        assert_eq!(cloud_reason(&s, true), Some("local_only"));
        assert!(ProviderId::Cloud.is_external() && !ProviderId::Cloud.needs_key());
        assert_eq!(serde_json::to_value(ProviderId::Cloud).unwrap(), "cloud");
    }

    #[test]
    fn http_errors_are_readable() {
        assert!(http_error("OpenAI", 401, "{}").to_string().contains("invalid API key"));
        assert!(http_error("Anthropic", 400, r#"{"error":{"message":"bad schema"}}"#)
            .to_string()
            .contains("bad schema"));
    }
}
