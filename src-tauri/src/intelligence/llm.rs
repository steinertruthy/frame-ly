//! Client of `framely-llm`, the local language-model process (see src-tauri/llm-worker). One
//! process per generation job: the model's memory is returned to the system when the job ends,
//! and cancelling kills the process immediately.

use crate::core::errors::{AppError, AppResult};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Event {
    #[serde(rename_all = "camelCase")]
    Loaded { ctx: u32 },
    Progress { phase: String, done: u32, total: u32 },
    Done { text: String, truncated: bool },
    Error { code: String, message: String },
}

#[derive(Debug, Clone)]
pub struct Output {
    pub text: String,
    pub truncated: bool,
}

/// Where the sidecar lives: next to the app executable (Tauri bundles it there).
pub fn worker_path() -> AppResult<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| AppError::Runtime("no executable dir".into()))?;
    let name = if cfg!(windows) { "framely-llm.exe" } else { "framely-llm" };
    let path = dir.join(name);
    if path.exists() {
        Ok(path)
    } else {
        Err(AppError::Runtime(format!("{} not found", path.display())))
    }
}

pub struct Worker {
    child: Arc<Mutex<Child>>,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    pub ctx: u32,
    watcher_stop: Arc<AtomicBool>,
}

impl Worker {
    /// Starts the process and loads `model`. `cancel` kills the process at any moment.
    pub fn start(worker: &Path, model: &Path, ctx: u32, gpu: bool, cancel: Arc<AtomicBool>) -> AppResult<Self> {
        let mut cmd = Command::new(worker);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd.spawn().map_err(|e| AppError::Runtime(format!("framely-llm: {e}")))?;
        let stdin = child.stdin.take().ok_or_else(|| AppError::Runtime("framely-llm stdin".into()))?;
        let stdout = BufReader::new(child.stdout.take().ok_or_else(|| AppError::Runtime("framely-llm stdout".into()))?);
        let child = Arc::new(Mutex::new(child));
        let watcher_stop = Arc::new(AtomicBool::new(false));
        {
            let (child, stop) = (child.clone(), watcher_stop.clone());
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    if cancel.load(Ordering::Relaxed) {
                        let _ = child.lock().unwrap().kill();
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });
        }
        let mut w = Self { child, stdin, stdout, ctx: 0, watcher_stop };
        w.send(json!({"op": "load", "model": model.to_string_lossy(), "ctx": ctx, "gpu": gpu}))?;
        match w.next(&mut |_, _, _| {})? {
            Event::Loaded { ctx } => w.ctx = ctx,
            Event::Error { code, message } => return Err(AppError::Platform(format!("language model ({code}): {message}"))),
            other => return Err(AppError::Runtime(format!("unexpected {other:?}"))),
        }
        Ok(w)
    }

    fn send(&mut self, v: Value) -> AppResult<()> {
        writeln!(self.stdin, "{v}").and_then(|_| self.stdin.flush()).map_err(|_| AppError::Cancelled)
    }

    fn next(&mut self, progress: &mut dyn FnMut(&str, u32, u32)) -> AppResult<Event> {
        loop {
            let mut line = String::new();
            // EOF: the process ended (cancelled or crashed).
            if self.stdout.read_line(&mut line)? == 0 {
                let status = self.child.lock().unwrap().try_wait().ok().flatten();
                return Err(match status {
                    Some(s) if s.success() => AppError::Runtime("framely-llm exited".into()),
                    _ => AppError::Cancelled,
                });
            }
            match serde_json::from_str::<Event>(&line) {
                Ok(Event::Progress { phase, done, total }) => progress(&phase, done, total),
                Ok(e) => return Ok(e),
                Err(_) => continue,
            }
        }
    }

    /// Generates a response constrained to `schema`. `progress(phase, done, total)` reports
    /// prompt evaluation and generated tokens. A prompt that doesn't fit returns
    /// `Provider("context_overflow")`.
    pub fn generate(
        &mut self,
        system: &str,
        user: &str,
        schema: &Value,
        max_tokens: u32,
        progress: &mut dyn FnMut(&str, u32, u32),
    ) -> AppResult<Output> {
        self.send(json!({"op": "generate", "system": system, "user": user, "schema": schema, "maxTokens": max_tokens, "temperature": 0.2}))?;
        match self.next(progress)? {
            Event::Done { text, truncated } => Ok(Output { text, truncated }),
            Event::Error { code, .. } if code == "context_overflow" => Err(AppError::Provider("context_overflow".into())),
            Event::Error { code, message } => Err(AppError::Platform(format!("language model ({code}): {message}"))),
            other => Err(AppError::Runtime(format!("unexpected {other:?}"))),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.watcher_stop.store(true, Ordering::Relaxed);
        let _ = self.send(json!({"op": "quit"}));
        let mut child = self.child.lock().unwrap();
        // Give it a moment to release the GPU cleanly, then make sure it's gone.
        for _ in 0..20 {
            if matches!(child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
    }
}
