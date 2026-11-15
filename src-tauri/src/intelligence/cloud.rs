//! FrameLy Cloud client: FrameLy's own AI service (cloud/ in this repository). The app never
//! talks to the AI provider behind it and holds no provider credentials.
//!
//! Identity: an anonymous Ed25519 key pair created on first use and kept in the OS keychain;
//! the device registers once (solving a small proof-of-work challenge) and signs every request
//! (method, path, timestamp, a unique request id and a hash of the body), so requests can't be
//! forged or replayed. The service builds the prompts itself from the structured request.

use super::{
    generation::Task,
    net,
    providers::{Input, Request, Runner},
};
use crate::core::{
    errors::{AppError, AppResult},
    transcript,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

const KEY_ACCOUNT: &str = "framely-cloud-device";

/// API address: `FRAMELY_CLOUD_URL` at run time (development) or at build time (releases).
pub fn base_url() -> Option<String> {
    std::env::var("FRAMELY_CLOUD_URL")
        .ok()
        .or_else(|| option_env!("FRAMELY_CLOUD_URL").map(str::to_string))
        .map(|u| u.trim_end_matches('/').to_string())
        .filter(|u| u.starts_with("https://") || u.starts_with("http://127.0.0.1") || u.starts_with("http://localhost"))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// "available" | "offline" | "unavailable" | "region_blocked" | "not_configured" | "local_only"
    pub state: String,
    /// Who processes the text behind FrameLy Cloud (e.g. "Google Gemini").
    pub processor: Option<String>,
    /// "may_train" (provider may use content to improve its products) | "no_training".
    pub data_use: Option<String>,
    pub max_input_chars: Option<u64>,
    pub requests_per_day: Option<u64>,
}

impl Status {
    fn only(state: &str) -> Self {
        Self { state: state.into(), processor: None, data_use: None, max_input_chars: None, requests_per_day: None }
    }
}

/// Connection check (no transcript involved). Short timeout: the UI shows "checking…" meanwhile.
pub fn status(local_only: bool) -> Status {
    let Some(base) = base_url() else { return Status::only("not_configured") };
    let agent = match net::agent(local_only, Some(Duration::from_secs(8))) {
        Ok(a) => a,
        Err(_) => return Status::only("local_only"),
    };
    let mut resp = match agent.get(&format!("{base}/v1/status")).call() {
        Ok(r) => r,
        Err(_) => return Status::only("offline"),
    };
    if resp.status().as_u16() != 200 {
        return Status::only("unavailable");
    }
    let Some(v) = resp.body_mut().read_to_string().ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else {
        return Status::only("unavailable");
    };
    let state = match (v["available"].as_bool(), v["reason"].as_str()) {
        (Some(true), _) => "available",
        (_, Some("region_blocked")) => "region_blocked",
        _ => "unavailable",
    };
    Status {
        state: state.into(),
        processor: v["processor"].as_str().map(str::to_string),
        data_use: v["dataUse"].as_str().map(str::to_string),
        max_input_chars: v["limits"]["maxInputChars"].as_u64(),
        requests_per_day: v["limits"]["requestsPerDay"].as_u64(),
    }
}

/// Maps the service's structured errors (`{error: {code, retryAfter}}`) to app errors.
pub fn map_error(status: u16, body: &str) -> AppError {
    let v: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let code = v["error"]["code"].as_str().unwrap_or("");
    let retry = v["error"]["retryAfter"].as_f64().map(|s| s.ceil() as u64);
    match code {
        "rate_limited" | "daily_limit" => AppError::CloudLimit { retry_after: retry },
        "service_busy" | "quota_exhausted" | "unavailable" => AppError::CloudUnavailable,
        "region_blocked" => AppError::CloudRegion,
        "unauthorized" => AppError::Runtime("cloud_unauthorized".into()),
        "too_large" => AppError::Provider("context_overflow".into()),
        "provider_error" | "invalid_request" => {
            AppError::Provider(format!("FrameLy Cloud: {}", v["error"]["message"].as_str().unwrap_or("request failed")))
        }
        _ if status >= 500 => AppError::CloudUnavailable,
        _ => AppError::Provider(format!("FrameLy Cloud: {}", v["error"]["message"].as_str().unwrap_or("request failed"))),
    }
}

/// Leading zero bits of a hash (proof of work).
pub fn leading_zero_bits(hash: &[u8]) -> u32 {
    let mut n = 0;
    for b in hash {
        if *b == 0 {
            n += 8;
        } else {
            return n + b.leading_zeros();
        }
    }
    n
}

pub fn solve_pow(challenge: &str, public_key: &str, bits: u32) -> u64 {
    (0u64..)
        .find(|n| leading_zero_bits(&Sha256::digest(format!("{challenge}:{public_key}:{n}"))) >= bits)
        .expect("a nonce exists")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The exact text signed for a request (must match cloud/src/auth.ts).
pub fn signing_string(method: &str, path: &str, timestamp: &str, request_id: &str, body: &str) -> String {
    format!("{method}\n{path}\n{timestamp}\n{request_id}\n{}", hex(&Sha256::digest(body.as_bytes())))
}

#[derive(Serialize, Deserialize)]
struct Stored {
    url: String,
    device_id: String,
    seed: String,
}

pub struct Device {
    id: String,
    key: SigningKey,
}

fn random<const N: usize>() -> AppResult<[u8; N]> {
    let mut b = [0u8; N];
    getrandom::fill(&mut b).map_err(|e| AppError::Platform(format!("random: {e}")))?;
    Ok(b)
}

/// The registered device for `base`, registering it on first use.
pub fn device(agent: &ureq::Agent, base: &str, force_new: bool) -> AppResult<Device> {
    if !force_new {
        if let Some(raw) = crate::platform::keystore::get(KEY_ACCOUNT)? {
            if let Ok(s) = serde_json::from_str::<Stored>(&raw) {
                if s.url == base {
                    if let Ok(seed) = B64.decode(&s.seed).map_err(|_| ()).and_then(|v| <[u8; 32]>::try_from(v).map_err(|_| ())) {
                        return Ok(Device { id: s.device_id, key: SigningKey::from_bytes(&seed) });
                    }
                }
            }
        }
    }
    let key = SigningKey::from_bytes(&random::<32>()?);
    let public_key = B64.encode(key.verifying_key().as_bytes());
    let mut r = agent.get(&format!("{base}/v1/devices/challenge")).call().map_err(net::net_err)?;
    let text = r.body_mut().read_to_string().map_err(net::net_err)?;
    if r.status().as_u16() != 200 {
        return Err(map_error(r.status().as_u16(), &text));
    }
    let c: Value = serde_json::from_str(&text)?;
    let challenge = c["challenge"].as_str().ok_or_else(|| AppError::Provider("FrameLy Cloud: bad challenge".into()))?.to_string();
    let nonce = solve_pow(&challenge, &public_key, c["bits"].as_u64().unwrap_or(20) as u32);
    let mut r = agent
        .post(&format!("{base}/v1/devices"))
        .header("content-type", "application/json")
        .send(json!({"challenge": challenge, "publicKey": public_key, "nonce": nonce.to_string()}).to_string())
        .map_err(net::net_err)?;
    let text = r.body_mut().read_to_string().map_err(net::net_err)?;
    if r.status().as_u16() != 200 {
        return Err(map_error(r.status().as_u16(), &text));
    }
    let id = serde_json::from_str::<Value>(&text)?["deviceId"]
        .as_str()
        .ok_or_else(|| AppError::Provider("FrameLy Cloud: bad registration".into()))?
        .to_string();
    let stored = Stored { url: base.into(), device_id: id.clone(), seed: B64.encode(key.to_bytes()) };
    crate::platform::keystore::set(KEY_ACCOUNT, &serde_json::to_string(&stored)?)?;
    Ok(Device { id, key })
}

/// Sends one signed request; returns `(status, body)`.
fn signed_post(agent: &ureq::Agent, base: &str, dev: &Device, path: &str, body: &str) -> AppResult<(u16, String)> {
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    let request_id = hex(&random::<16>()?);
    let sig = dev.key.sign(signing_string("POST", path, &timestamp, &request_id, body).as_bytes());
    let mut r = agent
        .post(&format!("{base}{path}"))
        .header("content-type", "application/json")
        .header("x-framely-device", &dev.id)
        .header("x-framely-timestamp", &timestamp)
        .header("x-framely-request", &request_id)
        .header("x-framely-signature", &B64.encode(sig.to_bytes()))
        .send(body.to_string())
        .map_err(net::net_err)?;
    let status = r.status().as_u16();
    Ok((status, r.body_mut().read_to_string().map_err(net::net_err)?))
}

/// The service's JSON body for a request (no prompt: the service writes it).
pub fn request_body(req: &Request<'_>) -> Value {
    let detail = super::generation::detail_name(req.detail);
    match &req.input {
        Input::Part { text, part } => json!({
            "language": req.language,
            "detail": detail,
            "transcript": text,
            "part": part.map(|(index, count)| json!({"index": index, "count": count})),
        }),
        Input::Combine { partials } => json!({
            "language": req.language,
            "detail": detail,
            "partials": partials.iter().map(|p| json!({
                "overview": p["overview"],
                "points": p["points"].as_array().into_iter().flatten().map(|x| json!({
                    "text": x["text"],
                    "time": x["ms"].as_u64().map(transcript::fmt_clock).unwrap_or_default(),
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        }),
    }
}

pub struct CloudRunner {
    pub base: String,
    pub agent: ureq::Agent,
    pub device: Device,
    pub max_input_chars: u64,
    pub cancel: Arc<AtomicBool>,
}

impl CloudRunner {
    pub fn open(local_only: bool, cancel: Arc<AtomicBool>) -> AppResult<Self> {
        let base = base_url().ok_or_else(|| AppError::Unsupported("cloud_not_configured".into()))?;
        let agent = net::agent(local_only, Some(Duration::from_secs(120)))?;
        let s = status(local_only);
        match s.state.as_str() {
            "available" => {}
            "region_blocked" => return Err(AppError::CloudRegion),
            "offline" => return Err(AppError::Network("FrameLy Cloud unreachable".into())),
            _ => return Err(AppError::CloudUnavailable),
        }
        let device = device(&agent, &base, false)?;
        Ok(Self { base, agent, device, max_input_chars: s.max_input_chars.unwrap_or(100_000), cancel })
    }

    /// Runs the request on a helper thread so cancelling returns at once (the reply is dropped).
    fn send(&mut self, path: &str, body: String) -> AppResult<String> {
        let mut retried = false;
        loop {
            let (tx, rx) = std::sync::mpsc::channel();
            let (agent, base, path_s, body_c) = (self.agent.clone(), self.base.clone(), path.to_string(), body.clone());
            let dev = Device { id: self.device.id.clone(), key: self.device.key.clone() };
            std::thread::spawn(move || {
                let _ = tx.send(signed_post(&agent, &base, &dev, &path_s, &body_c));
            });
            let (status, text) = loop {
                if self.cancel.load(Ordering::Relaxed) {
                    return Err(AppError::Cancelled);
                }
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(r) => break r?,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => return Err(AppError::Runtime("cloud request thread ended".into())),
                }
            };
            if status == 200 {
                let v: Value = serde_json::from_str(&text)?;
                return Ok(v["result"].to_string());
            }
            let err = map_error(status, &text);
            // The service may have forgotten this device (e.g. a reset): register again once.
            if matches!(&err, AppError::Runtime(m) if m == "cloud_unauthorized") && !retried {
                retried = true;
                self.device = device(&self.agent, &self.base, true)?;
                continue;
            }
            return Err(err);
        }
    }
}

impl Runner for CloudRunner {
    fn context_tokens(&self) -> u32 {
        // Input is limited in characters by the service; generation.rs budgets in tokens.
        (self.max_input_chars / 3) as u32 + 8_000
    }
    fn model_name(&self) -> String {
        "framely-cloud".into()
    }
    fn run(&mut self, _task: Task, _system: &str, _user: &str, _max: u32, _progress: &mut dyn FnMut(f64)) -> AppResult<String> {
        Err(AppError::Runtime("FrameLy Cloud receives structured requests".into()))
    }
    fn generate(&mut self, req: &Request<'_>, _max_tokens: u32, progress: &mut dyn FnMut(f64)) -> AppResult<String> {
        progress(0.0);
        let path = if req.task == Task::Summary { "/v1/ai/summarize" } else { "/v1/ai/topics" };
        let out = self.send(path, request_body(req).to_string());
        progress(1.0);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::settings::Detail;

    #[test]
    fn proof_of_work_and_signing_match_the_service() {
        assert_eq!(leading_zero_bits(&[0, 0b0001_0000, 0xff]), 11);
        let n = solve_pow("c", "pk", 8);
        assert!(leading_zero_bits(&Sha256::digest(format!("c:pk:{n}"))) >= 8);
        // Same layout as cloud/src/auth.ts signingString.
        let s = signing_string("POST", "/v1/ai/topics", "1700000000000", "abc", "{}");
        assert_eq!(s, format!("POST\n/v1/ai/topics\n1700000000000\nabc\n{}", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"));
    }

    #[test]
    fn errors_map_to_user_facing_states() {
        let e = |status, body: &str| map_error(status, body).code();
        assert_eq!(e(429, r#"{"error":{"code":"rate_limited","retryAfter":12}}"#), "cloud_limit");
        assert_eq!(e(429, r#"{"error":{"code":"daily_limit"}}"#), "cloud_limit");
        assert_eq!(e(503, r#"{"error":{"code":"quota_exhausted"}}"#), "cloud_unavailable");
        assert_eq!(e(503, r#"{"error":{"code":"service_busy"}}"#), "cloud_unavailable");
        assert_eq!(e(451, r#"{"error":{"code":"region_blocked"}}"#), "cloud_region");
        assert_eq!(e(502, r#"{"error":{"code":"provider_error","message":"x"}}"#), "provider");
        assert_eq!(e(500, "not json"), "cloud_unavailable");
        assert!(matches!(map_error(413, r#"{"error":{"code":"too_large"}}"#), AppError::Provider(m) if m == "context_overflow"));
    }

    #[test]
    fn requests_carry_structure_not_prompts() {
        let req = Request { task: Task::Summary, detail: Detail::Brief, language: Some("pt"), system: "SYS", input: Input::Part { text: "[00:01] oi", part: Some((1, 3)) } };
        let b = request_body(&req);
        assert_eq!(b, json!({"language": "pt", "detail": "brief", "transcript": "[00:01] oi", "part": {"index": 1, "count": 3}}));
        assert!(!b.to_string().contains("SYS"));
        let partials = vec![json!({"overview": "A", "points": [{"text": "x", "ms": 65_000}, {"text": "y", "ms": null}]})];
        let c = request_body(&Request { task: Task::Summary, detail: Detail::Balanced, language: None, system: "", input: Input::Combine { partials: &partials } });
        assert_eq!(c["partials"][0]["points"], json!([{"text": "x", "time": "01:05"}, {"text": "y", "time": ""}]));
    }

    /// Against a running service (`cd cloud && pnpm dev:mock`): registration, signed requests.
    /// `FRAMELY_CLOUD_URL=http://127.0.0.1:8787 cargo test -- --ignored cloud_round_trip`
    #[test]
    #[ignore = "needs a running FrameLy Cloud (wrangler dev)"]
    fn cloud_round_trip() {
        let cancel = Arc::new(AtomicBool::new(false));
        let mut r = CloudRunner::open(false, cancel).unwrap();
        let req = Request { task: Task::Summary, detail: Detail::Brief, language: Some("pt"), system: "", input: Input::Part { text: "[00:00] Bom dia.\n[00:05] Lançamento em dezembro.", part: None } };
        let out: Value = serde_json::from_str(&r.generate(&req, 1000, &mut |_| {}).unwrap()).unwrap();
        println!("{out}");
        assert!(out["overview"].as_str().unwrap().contains("Bom dia"));
        let topics = Request { task: Task::Topics, ..req };
        let t: Value = serde_json::from_str(&r.generate(&topics, 1000, &mut |_| {}).unwrap()).unwrap();
        assert!(t["topics"].is_array());
    }
}
