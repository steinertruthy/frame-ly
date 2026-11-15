//! `framely-llm`: FrameLy AI's local language-model runtime (llama.cpp).
//!
//! FrameLy starts this process on demand and talks to it over stdin/stdout, one JSON object per
//! line. Running apart from the app keeps llama.cpp's ggml away from whisper.cpp's (different
//! versions of the same library), frees the model's memory as soon as the work is done, makes
//! cancellation instant (the app ends the process) and keeps a model failure from taking the app
//! down.
//!
//! Requests:
//!   {"op":"load","model":PATH,"ctx":N,"gpu":BOOL,"threads":N}
//!   {"op":"count","text":STR}
//!   {"op":"generate","system":STR,"user":STR,"schema":JSON|null,"maxTokens":N,"temperature":F}
//! Events:
//!   {"type":"loaded","ctx":N,"trainCtx":N,"arch":STR,"gpu":BOOL}
//!   {"type":"count","tokens":N}
//!   {"type":"progress","phase":"prompt"|"generate","done":N,"total":N}
//!   {"type":"done","text":STR,"promptTokens":N,"genTokens":N,"promptMs":N,"genMs":N,"truncated":BOOL}
//!   {"type":"error","code":STR,"message":STR}

use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, LlamaModel},
    sampling::LlamaSampler,
    token::LlamaToken,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{BufRead, Write},
    num::NonZeroU32,
    time::Instant,
};

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum Request {
    Load { model: String, ctx: u32, gpu: bool, threads: Option<i32> },
    Count { text: String },
    #[serde(rename_all = "camelCase")]
    Generate { system: String, user: String, schema: Option<Value>, max_tokens: u32, temperature: Option<f32> },
    Quit,
}

fn emit(v: Value) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{v}");
    let _ = out.flush();
}

fn fail(code: &str, message: impl std::fmt::Display) {
    emit(json!({"type": "error", "code": code, "message": message.to_string()}));
}

/// Prompt format of the model family, with "thinking" turned off: summaries and topics need
/// the answer, and hidden reasoning would only cost time.
fn format_prompt(arch: &str, system: &str, user: &str) -> String {
    match arch {
        a if a.starts_with("gemma") => {
            format!("<|turn>system\n{system}<turn|>\n<|turn>user\n{user}<turn|>\n<|turn>model\n")
        }
        // Qwen (ChatML) and the default.
        _ => format!(
            "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n"
        ),
    }
}

struct Loaded {
    model: LlamaModel,
    ctx: u32,
    threads: i32,
    arch: String,
}

fn main() {
    let mut backend = match LlamaBackend::init() {
        Ok(b) => b,
        Err(e) => return fail("backend", e),
    };
    backend.void_logs();
    let mut loaded: Option<Loaded> = None;

    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                fail("bad_request", e);
                continue;
            }
        };
        match req {
            Request::Quit => break,
            Request::Load { model, ctx, gpu, threads } => {
                let params = LlamaModelParams::default().with_n_gpu_layers(if gpu { 999 } else { 0 });
                // Release the previous model before loading another.
                loaded = None;
                match LlamaModel::load_from_file(&backend, &model, &params) {
                    Ok(m) => {
                        let arch = m.meta_val_str("general.architecture").unwrap_or_default();
                        let train = m.n_ctx_train();
                        let ctx = ctx.min(train).max(2048);
                        let threads = threads.unwrap_or_else(|| {
                            std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4).clamp(1, 8)
                        });
                        emit(json!({"type": "loaded", "ctx": ctx, "trainCtx": train, "arch": arch, "gpu": gpu}));
                        loaded = Some(Loaded { model: m, ctx, threads, arch });
                    }
                    Err(e) => fail("load", e),
                }
            }
            Request::Count { text } => match &loaded {
                Some(l) => {
                    let n = l.model.vocab().tokenize(text.as_bytes(), false, false).len();
                    emit(json!({"type": "count", "tokens": n}));
                }
                None => fail("not_loaded", "no model loaded"),
            },
            Request::Generate { system, user, schema, max_tokens, temperature } => match &loaded {
                Some(l) => {
                    if let Err((code, msg)) = generate(&backend, l, &system, &user, schema.as_ref(), max_tokens, temperature.unwrap_or(0.2)) {
                        fail(code, msg);
                    }
                }
                None => fail("not_loaded", "no model loaded"),
            },
        }
    }
    // Free GPU resources in order (model, then backend) before the process exits.
    drop(loaded);
    drop(backend);
}

type GenError = (&'static str, String);

fn generate(
    backend: &LlamaBackend,
    l: &Loaded,
    system: &str,
    user: &str,
    schema: Option<&Value>,
    max_tokens: u32,
    temperature: f32,
) -> Result<(), GenError> {
    let model = &l.model;
    let vocab = model.vocab();
    let prompt = format_prompt(&l.arch, system, user);
    let tokens: Vec<LlamaToken> = vocab.tokenize(prompt.as_bytes(), true, true);
    let total = tokens.len() as u32;
    if total + max_tokens > l.ctx {
        return Err(("context_overflow", format!("{{\"promptTokens\":{total},\"ctx\":{}}}", l.ctx)));
    }
    let n_batch = 512u32;
    let params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(l.ctx))
        .with_n_batch(n_batch)
        .with_n_threads(l.threads)
        .with_n_threads_batch(l.threads);
    let mut ctx = model.new_context(backend, params).map_err(|e| ("context", e.to_string()))?;

    let mut sampler = {
        let mut chain = Vec::new();
        if let Some(schema) = schema {
            let grammar = llama_cpp_2::json_schema_to_grammar(&schema.to_string()).map_err(|e| ("schema", e.to_string()))?;
            chain.push(LlamaSampler::grammar(model, &grammar, "root").map_err(|e| ("grammar", e.to_string()))?);
        }
        chain.push(LlamaSampler::top_k(40));
        chain.push(LlamaSampler::top_p(0.9, 1));
        chain.push(LlamaSampler::temp(temperature));
        chain.push(LlamaSampler::dist(1234));
        LlamaSampler::chain_simple(chain)
    };

    // Prompt, in batches (progress is real: tokens evaluated so far).
    let started = Instant::now();
    let mut batch = LlamaBatch::new(n_batch as usize, 1);
    let mut pos = 0i32;
    for chunk in tokens.chunks(n_batch as usize) {
        batch.clear();
        for (i, t) in chunk.iter().enumerate() {
            let last = pos as usize + i + 1 == tokens.len();
            batch.add(*t, pos + i as i32, &[0], last).map_err(|e| ("batch", e.to_string()))?;
        }
        ctx.decode(&mut batch).map_err(|e| ("decode", e.to_string()))?;
        pos += chunk.len() as i32;
        emit(json!({"type": "progress", "phase": "prompt", "done": pos, "total": total}));
    }
    let prompt_ms = started.elapsed().as_millis() as u64;

    // Generation.
    let gen_started = Instant::now();
    let mut out: Vec<u8> = Vec::new();
    let mut generated = 0u32;
    let mut truncated = true;
    let mut last_index = batch.n_tokens() - 1;
    while generated < max_tokens {
        let token = sampler.sample(&ctx, last_index);
        if vocab.is_eog(token) {
            truncated = false;
            break;
        }
        out.extend(vocab.token_to_piece(token, false, None));
        generated += 1;
        if generated.is_multiple_of(16) {
            emit(json!({"type": "progress", "phase": "generate", "done": generated, "total": max_tokens}));
        }
        batch.clear();
        batch.add(token, pos, &[0], true).map_err(|e| ("batch", e.to_string()))?;
        pos += 1;
        last_index = 0;
        ctx.decode(&mut batch).map_err(|e| ("decode", e.to_string()))?;
    }
    let text = String::from_utf8_lossy(&out).into_owned();
    emit(json!({
        "type": "done",
        "text": text,
        "promptTokens": total,
        "genTokens": generated,
        "promptMs": prompt_ms,
        "genMs": gen_started.elapsed().as_millis() as u64,
        "truncated": truncated,
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::format_prompt;

    #[test]
    fn prompt_formats_disable_thinking() {
        let q = format_prompt("qwen35", "S", "U");
        assert!(q.starts_with("<|im_start|>system\nS<|im_end|>") && q.ends_with("<think>\n\n</think>\n\n"));
        let g = format_prompt("gemma4", "S", "U");
        assert_eq!(g, "<|turn>system\nS<turn|>\n<|turn>user\nU<turn|>\n<|turn>model\n");
    }

    #[test]
    fn requests_parse() {
        let r: super::Request = serde_json::from_str(r#"{"op":"generate","system":"s","user":"u","schema":null,"maxTokens":10}"#).unwrap();
        assert!(matches!(r, super::Request::Generate { max_tokens: 10, .. }));
    }
}
