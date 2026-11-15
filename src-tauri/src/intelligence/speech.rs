//! Speech recognition with whisper.cpp, in blocks of about five minutes cut at the quietest
//! moment (so long recordings stay bounded in memory and an interrupted job resumes at the next
//! block). Each finished block is handed to `on_block` to be persisted before the next one starts.

use crate::core::{
    errors::{AppError, AppResult},
    settings::Precision,
    transcript::{self, Segment, Word},
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const RATE: usize = 16_000;
const BLOCK: usize = 300 * RATE;
const SEARCH: usize = 20 * RATE;
const FRAME: usize = RATE / 10;

pub struct Request<'a> {
    pub pcm: &'a Path,
    pub samples: u64,
    pub model: &'a Path,
    pub precision: Precision,
    /// ISO code, or None to detect it.
    pub language: Option<String>,
    /// First block to process (resume).
    pub start_block: u32,
    /// Text of the last processed block, for continuity.
    pub previous_text: String,
    pub gpu: bool,
    pub cancel: Arc<AtomicBool>,
}

pub struct BlockResult {
    pub index: u32,
    pub segments: Vec<Segment>,
    pub done_ms: u64,
    pub language: Option<String>,
}

/// Per-frame energy of the PCM file (one value per 100 ms), streamed.
fn frame_energy(pcm: &Path) -> AppResult<Vec<f32>> {
    let mut f = std::io::BufReader::new(File::open(pcm)?);
    let mut buf = vec![0u8; FRAME * 4];
    let mut out = Vec::new();
    loop {
        let mut filled = 0;
        while filled < buf.len() {
            let n = f.read(&mut buf[filled..])?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        if filled < 4 {
            break;
        }
        let n = filled / 4;
        let e = buf[..n * 4].chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]).powi(2)).sum::<f32>() / n as f32;
        out.push(e);
        if filled < buf.len() {
            break;
        }
    }
    Ok(out)
}

/// Block start samples for the whole file.
pub fn plan_blocks(pcm: &Path, samples: u64) -> AppResult<Vec<usize>> {
    let energy = frame_energy(pcm)?;
    let frames = transcript::block_starts(energy.len(), BLOCK / FRAME, SEARCH / FRAME, 1, |a, _| energy[a]);
    let mut starts: Vec<usize> = frames.into_iter().map(|f| f * FRAME).collect();
    starts.retain(|&s| (s as u64) < samples.max(1));
    if starts.is_empty() {
        starts.push(0);
    }
    Ok(starts)
}

fn read_samples(pcm: &Path, from: usize, to: usize) -> AppResult<Vec<f32>> {
    let mut f = File::open(pcm)?;
    f.seek(SeekFrom::Start(from as u64 * 4))?;
    let mut bytes = vec![0u8; (to - from) * 4];
    f.read_exact(&mut bytes)?;
    Ok(bytes.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
}

fn is_special(text: &str) -> bool {
    let t = text.trim();
    (t.starts_with("[_") && t.ends_with(']')) || t.starts_with("<|")
}

/// Merges sub-word tokens into words with timing and the lowest token confidence.
fn words_of(seg: &whisper_rs::WhisperSegment<'_>, offset_ms: u64) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();
    for i in 0..seg.n_tokens() {
        let Some(tok) = seg.get_token(i) else { continue };
        let Ok(text) = tok.to_str_lossy() else { continue };
        if is_special(&text) || text.is_empty() {
            continue;
        }
        let d = tok.token_data();
        let (a, b) = (offset_ms + d.t0.max(0) as u64 * 10, offset_ms + d.t1.max(0) as u64 * 10);
        let starts_word = text.starts_with(' ') || words.is_empty();
        if starts_word {
            words.push(Word { text: text.trim_start().to_string(), start_ms: a, end_ms: b.max(a), p: d.p });
        } else if let Some(w) = words.last_mut() {
            w.text.push_str(&text);
            w.end_ms = b.max(w.end_ms);
            w.p = w.p.min(d.p);
        }
    }
    words.retain(|w| !w.text.trim().is_empty());
    words
}

fn params<'a>(precision: Precision, threads: i32) -> FullParams<'a, 'a> {
    let mut p = match precision {
        Precision::Max => FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: -1.0 }),
        Precision::Fast => FullParams::new(SamplingStrategy::Greedy { best_of: 1 }),
        _ => FullParams::new(SamplingStrategy::Greedy { best_of: 3 }),
    };
    p.set_n_threads(threads);
    p.set_print_progress(false);
    p.set_print_realtime(false);
    p.set_print_timestamps(false);
    p.set_print_special(false);
    p.set_token_timestamps(true);
    p.set_suppress_blank(true);
    p.set_suppress_nst(true);
    p
}

/// Loads the model (`on_loaded` once ready) and transcribes every remaining block.
/// `on_progress` gets the overall fraction (0…1) of audio processed.
pub fn run(
    req: Request<'_>,
    on_loaded: impl FnOnce(),
    on_progress: Arc<dyn Fn(f64) + Send + Sync>,
    mut on_block: impl FnMut(BlockResult) -> AppResult<()>,
) -> AppResult<()> {
    let starts = plan_blocks(req.pcm, req.samples)?;
    let total = req.samples as usize;
    let mut cparams = WhisperContextParameters::default();
    cparams.use_gpu(req.gpu);
    let ctx = WhisperContext::new_with_params(req.model, cparams)
        .map_err(|e| AppError::Platform(format!("speech model: {e}")))?;
    on_loaded();
    let threads = std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(4).clamp(1, 8);
    let mut language = req.language.clone();
    let mut previous = req.previous_text.clone();

    for (index, &from) in starts.iter().enumerate().skip(req.start_block as usize) {
        if req.cancel.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }
        let to = starts.get(index + 1).copied().unwrap_or(total).min(total);
        if to <= from {
            continue;
        }
        let audio = read_samples(req.pcm, from, to)?;
        let mut state = ctx.create_state().map_err(|e| AppError::Platform(format!("speech state: {e}")))?;
        let mut p = params(req.precision, threads);
        let lang = language.clone().unwrap_or_else(|| "auto".into());
        p.set_language(Some(&lang));
        // A short tail of the previous block keeps names and spelling consistent.
        let prompt: String = previous.chars().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect();
        if !prompt.trim().is_empty() {
            p.set_initial_prompt(&prompt);
        }
        let progress = on_progress.clone();
        let (block_from, block_len) = (from as f64, (to - from) as f64);
        p.set_progress_callback_safe(move |pct: i32| {
            progress(((block_from + block_len * pct as f64 / 100.0) / total as f64).min(1.0));
        });
        let cancel = req.cancel.clone();
        // whisper-rs 0.16 casts the callback's user data to the closure type it was given but
        // stores it boxed as `dyn FnMut`; handing it an already boxed `dyn FnMut` makes the two
        // agree (an unboxed closure reads garbage and aborts every run).
        let abort: Box<dyn FnMut() -> bool> = Box::new(move || cancel.load(Ordering::Relaxed));
        p.set_abort_callback_safe(abort);
        let result = state.full(p, &audio);
        if req.cancel.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }
        result.map_err(|e| AppError::Platform(format!("speech recognition: {e}")))?;

        if language.is_none() {
            language = whisper_rs::get_lang_str(state.full_lang_id_from_state()).map(str::to_string);
        }
        let offset_ms = (from as u64 * 1000) / RATE as u64;
        let mut segments = Vec::new();
        for seg in state.as_iter() {
            let text = seg.to_str_lossy().map(|t| t.trim().to_string()).unwrap_or_default();
            // Skip silence the model filled with nothing, or with a likely hallucination.
            if text.is_empty() || is_special(&text) || (seg.no_speech_probability() > 0.8 && text.len() < 40) {
                continue;
            }
            let a = offset_ms + seg.start_timestamp().max(0) as u64 * 10;
            let b = offset_ms + seg.end_timestamp().max(0) as u64 * 10;
            segments.push(Segment {
                idx: 0,
                start_ms: a,
                end_ms: b.max(a),
                text,
                words: words_of(&seg, offset_ms),
                block: index as u32,
                edited: false,
            });
        }
        previous = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
        on_block(BlockResult {
            index: index as u32,
            segments,
            done_ms: (to as u64 * 1000) / RATE as u64,
            language: language.clone(),
        })?;
        on_progress((to as f64 / total as f64).min(1.0));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_blocks_at_quiet_points() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a.pcm");
        // 12 minutes of "speech" (noise) with a silent second at 5:10.
        let n = 12 * 60 * RATE;
        let mut bytes = Vec::with_capacity(n * 4);
        for i in 0..n {
            let silent = (310 * RATE..311 * RATE).contains(&i);
            let v: f32 = if silent { 0.0 } else { ((i * 7919) % 1000) as f32 / 1000.0 - 0.5 };
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        std::fs::write(&path, &bytes).unwrap();
        let starts = plan_blocks(&path, n as u64).unwrap();
        assert_eq!(starts[0], 0);
        assert!(starts.len() >= 2);
        let cut = starts[1] as f64 / RATE as f64;
        assert!((310.0..=311.0).contains(&cut), "cut at the silence, got {cut}s");
        assert_eq!(read_samples(&path, 0, 3).unwrap().len(), 3);
    }

    /// Real recognition with an installed model: `FRAMELY_TEST_SPEECH_MODEL=<ggml file>` and
    /// `FRAMELY_TEST_PCM=<16 kHz mono f32 file>` (see docs/development.md).
    #[test]
    #[ignore = "needs a speech model and audio on disk"]
    fn recognizes_real_speech() {
        let model = std::env::var("FRAMELY_TEST_SPEECH_MODEL").unwrap();
        let pcm = std::env::var("FRAMELY_TEST_PCM").unwrap();
        let samples = std::fs::metadata(&pcm).unwrap().len() / 4;
        let mut blocks = Vec::new();
        run(
            Request {
                pcm: Path::new(&pcm),
                samples,
                model: Path::new(&model),
                precision: Precision::Balanced,
                language: None,
                start_block: 0,
                previous_text: String::new(),
                gpu: cfg!(target_os = "macos"),
                cancel: Arc::new(AtomicBool::new(false)),
            },
            || {},
            Arc::new(|_| {}),
            |b| {
                blocks.push(b);
                Ok(())
            },
        )
        .unwrap();
        let text: String = blocks.iter().flat_map(|b| b.segments.iter().map(|s| s.text.clone())).collect::<Vec<_>>().join(" ");
        println!("{:?} {text}", blocks[0].language);
        assert!(!text.is_empty());
        assert!(blocks.iter().all(|b| b.segments.iter().all(|s| s.end_ms >= s.start_ms)));
    }
}
