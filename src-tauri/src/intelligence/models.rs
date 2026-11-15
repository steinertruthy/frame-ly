//! Model manager: downloads (resumable with HTTP Range, pausable, cancellable), SHA-256
//! verification, free-space checks, versions and updates, corruption detection and removal.
//! Files live in `<app data>/models`; `manifest.json` records what was installed and verified.

use super::{
    catalog::{self, ModelSpec},
    net,
};
use crate::core::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Installed {
    version: u32,
    file: String,
    size: u64,
    sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Missing,
    Downloading,
    Verifying,
    Paused,
    Installed,
    /// Installed, and a newer version exists.
    Update,
    Corrupt,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    #[serde(flatten)]
    pub spec: ModelSpec,
    pub state: State,
    pub received: u64,
    pub installed_version: Option<u32>,
    /// Bytes on disk (installed file and partial download).
    pub disk_bytes: u64,
    pub error: Option<String>,
    pub in_use: bool,
}

struct Download {
    received: AtomicU64,
    pause: AtomicBool,
    cancel: AtomicBool,
    verifying: AtomicBool,
}

pub type Notify = Arc<dyn Fn(ModelStatus) + Send + Sync>;

pub struct ModelManager {
    dir: PathBuf,
    manifest: Mutex<HashMap<String, Installed>>,
    downloads: Mutex<HashMap<&'static str, Arc<Download>>>,
    errors: Mutex<HashMap<&'static str, String>>,
    corrupt: Mutex<HashSet<&'static str>>,
    in_use: Mutex<HashMap<&'static str, usize>>,
    notify: Mutex<Option<Notify>>,
}

fn part_path(dir: &Path, spec: &ModelSpec) -> PathBuf {
    dir.join(format!("{}.part", spec.file))
}

/// SHA-256 of a file, streamed.
pub fn sha256_file(path: &Path, cancel: Option<&AtomicBool>) -> AppResult<String> {
    let mut f = fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
            return Err(AppError::Cancelled);
        }
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

impl ModelManager {
    pub fn new(dir: PathBuf) -> Self {
        let manifest = fs::read(dir.join("manifest.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            dir,
            manifest: Mutex::new(manifest),
            downloads: Mutex::default(),
            errors: Mutex::default(),
            corrupt: Mutex::default(),
            in_use: Mutex::default(),
            notify: Mutex::new(None),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn set_notify(&self, n: Notify) {
        *self.notify.lock().unwrap() = Some(n);
    }

    fn emit(&self, spec: &'static ModelSpec) {
        let n = self.notify.lock().unwrap().clone();
        if let Some(n) = n {
            n(self.status_of(spec));
        }
    }

    fn save_manifest(&self) -> AppResult<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join("manifest.json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&*self.manifest.lock().unwrap())?)?;
        fs::rename(tmp, self.dir.join("manifest.json"))?;
        Ok(())
    }

    /// Path of a usable installed model (any version), checked cheaply (presence and size).
    pub fn ready_path(&self, spec: &ModelSpec) -> Option<PathBuf> {
        let m = self.manifest.lock().unwrap();
        let inst = m.get(spec.id)?;
        let path = self.dir.join(&inst.file);
        let ok = fs::metadata(&path).map(|md| md.len() == inst.size).unwrap_or(false);
        (ok && !self.corrupt.lock().unwrap().contains(spec.id)).then_some(path)
    }

    pub fn status_of(&self, spec: &'static ModelSpec) -> ModelStatus {
        let installed = self.manifest.lock().unwrap().get(spec.id).cloned();
        let dl = self.downloads.lock().unwrap().get(spec.id).cloned();
        let part = fs::metadata(part_path(&self.dir, spec)).map(|m| m.len()).unwrap_or(0);
        let file_len = installed.as_ref().and_then(|i| fs::metadata(self.dir.join(&i.file)).ok()).map(|m| m.len());
        let size_ok = installed.as_ref().zip(file_len).is_some_and(|(i, l)| i.size == l);
        let corrupt = self.corrupt.lock().unwrap().contains(spec.id) || (installed.is_some() && !size_ok && file_len.is_some());
        let state = if let Some(d) = &dl {
            if d.verifying.load(Ordering::Relaxed) {
                State::Verifying
            } else {
                State::Downloading
            }
        } else if corrupt {
            State::Corrupt
        } else if installed.is_some() && size_ok {
            if installed.as_ref().unwrap().version < spec.version {
                State::Update
            } else {
                State::Installed
            }
        } else if part > 0 {
            State::Paused
        } else {
            State::Missing
        };
        ModelStatus {
            spec: *spec,
            state,
            received: dl.map(|d| d.received.load(Ordering::Relaxed)).unwrap_or(part),
            installed_version: installed.as_ref().filter(|_| size_ok).map(|i| i.version),
            disk_bytes: file_len.unwrap_or(0) + part,
            error: self.errors.lock().unwrap().get(spec.id).cloned(),
            in_use: self.in_use.lock().unwrap().get(spec.id).copied().unwrap_or(0) > 0,
        }
    }

    pub fn all(&self) -> Vec<ModelStatus> {
        catalog::MODELS.iter().map(|s| self.status_of(s)).collect()
    }

    /// Marks a model as in use for the lifetime of the guard (it can't be removed meanwhile).
    pub fn use_model(self: &Arc<Self>, spec: &'static ModelSpec) -> InUse {
        *self.in_use.lock().unwrap().entry(spec.id).or_default() += 1;
        InUse { mgr: self.clone(), id: spec.id }
    }

    /// Starts or resumes a download on a background thread.
    pub fn download(self: &Arc<Self>, spec: &'static ModelSpec, local_only: bool) -> AppResult<()> {
        let agent = net::agent(local_only, None)?;
        let mut downloads = self.downloads.lock().unwrap();
        if downloads.contains_key(spec.id) {
            return Ok(());
        }
        fs::create_dir_all(&self.dir)?;
        let part = part_path(&self.dir, spec);
        let have = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let needed = spec.size.saturating_sub(have);
        if let Some(free) = crate::platform::disk::free_bytes(&self.dir) {
            // Keep a margin so the disk never fills completely.
            if free < needed + 512 * 1024 * 1024 {
                return Err(AppError::DiskFull);
            }
        }
        let d = Arc::new(Download {
            received: AtomicU64::new(have),
            pause: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            verifying: AtomicBool::new(false),
        });
        downloads.insert(spec.id, d.clone());
        drop(downloads);
        self.errors.lock().unwrap().remove(spec.id);
        self.emit(spec);
        let mgr = self.clone();
        std::thread::Builder::new()
            .name(format!("download-{}", spec.id))
            .spawn(move || {
                let result = mgr.run_download(&agent, spec, &d);
                mgr.downloads.lock().unwrap().remove(spec.id);
                match result {
                    Ok(()) => tracing::info!(model = spec.id, "model installed"),
                    Err(AppError::Cancelled) if d.pause.load(Ordering::Relaxed) => {}
                    Err(AppError::Cancelled) => {
                        let _ = fs::remove_file(part_path(&mgr.dir, spec));
                    }
                    Err(e) => {
                        tracing::warn!(model = spec.id, error = %e, "model download failed");
                        mgr.errors.lock().unwrap().insert(spec.id, e.code().to_string());
                    }
                }
                mgr.emit(spec);
            })
            .map_err(|e| AppError::Runtime(e.to_string()))?;
        Ok(())
    }

    fn run_download(&self, agent: &ureq::Agent, spec: &'static ModelSpec, d: &Download) -> AppResult<()> {
        let part = part_path(&self.dir, spec);
        let have = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        if have < spec.size {
            let mut req = agent.get(spec.url);
            if have > 0 {
                req = req.header("Range", &format!("bytes={have}-"));
            }
            let resp = req.call().map_err(net::net_err)?;
            let status = resp.status().as_u16();
            let append = match status {
                206 => true,
                200 => false,
                416 if have > 0 => true,
                _ => return Err(AppError::Network(format!("HTTP {status}"))),
            };
            let mut file = fs::OpenOptions::new().create(true).append(append).write(true).truncate(!append).open(&part)?;
            if !append {
                d.received.store(0, Ordering::Relaxed);
            }
            if status != 416 {
                let mut body = resp.into_body();
                let mut reader = body.as_reader();
                let mut buf = vec![0u8; 1 << 20];
                let mut last = Instant::now();
                loop {
                    if d.cancel.load(Ordering::Relaxed) || d.pause.load(Ordering::Relaxed) {
                        file.flush()?;
                        return Err(AppError::Cancelled);
                    }
                    let n = reader.read(&mut buf).map_err(|e| AppError::Network(e.to_string()))?;
                    if n == 0 {
                        break;
                    }
                    file.write_all(&buf[..n])?;
                    let total = d.received.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
                    if total > spec.size {
                        drop(file);
                        let _ = fs::remove_file(&part);
                        return Err(AppError::Integrity("larger than expected".into()));
                    }
                    if last.elapsed() >= Duration::from_millis(250) {
                        last = Instant::now();
                        self.emit(spec);
                    }
                }
                file.flush()?;
            }
        }
        let got = fs::metadata(&part)?.len();
        if got != spec.size {
            return Err(AppError::Network(format!("incomplete download ({got} of {} bytes)", spec.size)));
        }
        d.verifying.store(true, Ordering::Relaxed);
        self.emit(spec);
        let hash = sha256_file(&part, Some(&d.cancel))?;
        if hash != spec.sha256 {
            let _ = fs::remove_file(&part);
            return Err(AppError::Integrity(spec.id.into()));
        }
        let dest = self.dir.join(spec.file);
        fs::rename(&part, &dest)?;
        let previous = self.manifest.lock().unwrap().insert(
            spec.id.to_string(),
            Installed { version: spec.version, file: spec.file.into(), size: spec.size, sha256: hash },
        );
        if let Some(prev) = previous.filter(|p| p.file != spec.file) {
            let _ = fs::remove_file(self.dir.join(prev.file));
        }
        self.corrupt.lock().unwrap().remove(spec.id);
        self.save_manifest()
    }

    pub fn pause(&self, spec: &'static ModelSpec) {
        if let Some(d) = self.downloads.lock().unwrap().get(spec.id) {
            d.pause.store(true, Ordering::Relaxed);
        }
    }

    /// Stops a download and deletes the partial file.
    pub fn cancel(&self, spec: &'static ModelSpec) {
        if let Some(d) = self.downloads.lock().unwrap().get(spec.id) {
            d.cancel.store(true, Ordering::Relaxed);
            return;
        }
        let _ = fs::remove_file(part_path(&self.dir, spec));
        self.errors.lock().unwrap().remove(spec.id);
        self.emit(spec);
    }

    pub fn remove(&self, spec: &'static ModelSpec) -> AppResult<()> {
        if self.in_use.lock().unwrap().get(spec.id).copied().unwrap_or(0) > 0 {
            return Err(AppError::Runtime("model in use".into()));
        }
        self.cancel(spec);
        if let Some(inst) = self.manifest.lock().unwrap().remove(spec.id) {
            let _ = fs::remove_file(self.dir.join(inst.file));
        }
        self.corrupt.lock().unwrap().remove(spec.id);
        self.save_manifest()?;
        self.emit(spec);
        Ok(())
    }

    /// Re-hashes an installed model; a mismatch marks it corrupt (download again to repair).
    pub fn verify(&self, spec: &'static ModelSpec) -> AppResult<bool> {
        let inst = self.manifest.lock().unwrap().get(spec.id).cloned().ok_or_else(|| AppError::ModelMissing(spec.id.into()))?;
        let ok = sha256_file(&self.dir.join(&inst.file), None).map(|h| h == inst.sha256).unwrap_or(false);
        if ok {
            self.corrupt.lock().unwrap().remove(spec.id);
        } else {
            self.corrupt.lock().unwrap().insert(spec.id);
        }
        self.emit(spec);
        Ok(ok)
    }

    /// Repairs a corrupt model: drops the bad file and downloads it again.
    pub fn repair(self: &Arc<Self>, spec: &'static ModelSpec, local_only: bool) -> AppResult<()> {
        if let Some(inst) = self.manifest.lock().unwrap().remove(spec.id) {
            let _ = fs::remove_file(self.dir.join(inst.file));
        }
        self.corrupt.lock().unwrap().remove(spec.id);
        self.save_manifest()?;
        self.download(spec, local_only)
    }

    pub fn disk_usage(&self) -> u64 {
        fs::read_dir(&self.dir)
            .map(|rd| rd.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum())
            .unwrap_or(0)
    }
}

pub struct InUse {
    mgr: Arc<ModelManager>,
    id: &'static str,
}

impl Drop for InUse {
    fn drop(&mut self) {
        if let Some(n) = self.mgr.in_use.lock().unwrap().get_mut(self.id) {
            *n = n.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::catalog::Kind;
    use std::{io::BufRead, net::TcpListener};

    /// Minimal HTTP/1.1 server with Range support serving `body`.
    fn serve(body: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
                let mut range_from = None;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    if let Some(v) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                        range_from = v.trim().trim_end_matches('-').parse::<usize>().ok();
                    }
                }
                let mut s = stream;
                let (status, slice) = match range_from {
                    Some(from) => ("206 Partial Content", &body[from..]),
                    None => ("200 OK", &body[..]),
                };
                let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", slice.len());
                let _ = s.write_all(slice);
            }
        });
        format!("http://{addr}/model.bin")
    }

    fn spec_for(body: &[u8], url: String, sha: Option<String>) -> &'static ModelSpec {
        let sha = sha.unwrap_or_else(|| Sha256::digest(body).iter().map(|b| format!("{b:02x}")).collect());
        Box::leak(Box::new(ModelSpec {
            id: "speech-fast",
            kind: Kind::Speech,
            version: 1,
            file: "test.v1.bin",
            url: Box::leak(url.into_boxed_str()),
            size: body.len() as u64,
            sha256: Box::leak(sha.into_boxed_str()),
            license: "MIT",
            ram_mb: 1,
            context: 0,
        }))
    }

    fn wait_idle(m: &ModelManager, spec: &'static ModelSpec) -> ModelStatus {
        for _ in 0..500 {
            let s = m.status_of(spec);
            if !matches!(s.state, State::Downloading | State::Verifying) {
                return s;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("download did not finish");
    }

    #[test]
    fn downloads_resumes_and_verifies() {
        let tmp = tempfile::tempdir().unwrap();
        let body: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        let spec = spec_for(&body, serve(body.clone()), None);
        let m = Arc::new(ModelManager::new(tmp.path().join("models")));
        // A previous session left half of the file.
        fs::create_dir_all(m.dir()).unwrap();
        fs::write(part_path(m.dir(), spec), &body[..1_234_567]).unwrap();
        assert_eq!(m.status_of(spec).state, State::Paused);
        m.download(spec, false).unwrap();
        let s = wait_idle(&m, spec);
        assert_eq!(s.state, State::Installed, "{:?}", s.error);
        assert_eq!(fs::read(m.ready_path(spec).unwrap()).unwrap(), body);
        // Survives a restart (manifest).
        let again = ModelManager::new(tmp.path().join("models"));
        assert!(again.ready_path(spec).is_some());
        assert!(again.verify(spec).unwrap());
        // Damage on disk is detected.
        fs::write(m.dir().join(spec.file), vec![0u8; body.len()]).unwrap();
        assert!(!again.verify(spec).unwrap());
        assert_eq!(again.status_of(spec).state, State::Corrupt);
        assert!(again.ready_path(spec).is_none());
        let again = Arc::new(again);
        again.remove(spec).unwrap();
        assert_eq!(again.status_of(spec).state, State::Missing);
    }

    #[test]
    fn rejects_a_wrong_hash_and_local_only() {
        let tmp = tempfile::tempdir().unwrap();
        let body = vec![7u8; 100_000];
        let spec = spec_for(&body, serve(body.clone()), Some("0".repeat(64)));
        let m = Arc::new(ModelManager::new(tmp.path().to_path_buf()));
        assert_eq!(m.download(spec, true).unwrap_err().code(), "local_only");
        m.download(spec, false).unwrap();
        let s = wait_idle(&m, spec);
        assert_eq!((s.state, s.error.as_deref()), (State::Missing, Some("integrity")));
        assert!(!part_path(m.dir(), spec).exists(), "bad data is not kept");
    }

    #[test]
    fn models_in_use_cannot_be_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let m = Arc::new(ModelManager::new(tmp.path().to_path_buf()));
        let spec = catalog::spec("speech-fast").unwrap();
        let guard = m.use_model(spec);
        assert!(m.status_of(spec).in_use);
        assert!(m.remove(spec).is_err());
        drop(guard);
        assert!(m.remove(spec).is_ok());
    }
}
