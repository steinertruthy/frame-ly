//! Transcript model shared by speech recognition, storage, the UI and exports: timed segments
//! (with optional word-level timing and confidence), paragraph grouping, block splitting for long
//! audio, and text exports (TXT, SRT, VTT, Markdown). Pure and tested.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    /// Model confidence, 0–1.
    pub p: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub idx: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    #[serde(default)]
    pub words: Vec<Word>,
    /// Processing block the segment came from (resume unit).
    #[serde(default)]
    pub block: u32,
    #[serde(default)]
    pub edited: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paragraph {
    pub start_ms: u64,
    pub end_ms: u64,
    /// Indices into the segment list.
    pub segments: Vec<u32>,
}

/// A pause longer than this starts a new paragraph.
const PARAGRAPH_GAP_MS: u64 = 1_600;
/// Paragraphs are closed at the next sentence end once they reach this many characters.
const PARAGRAPH_SOFT_CHARS: usize = 420;
/// …and always closed at this size.
const PARAGRAPH_HARD_CHARS: usize = 900;

fn ends_sentence(text: &str) -> bool {
    matches!(text.trim_end().chars().last(), Some('.' | '!' | '?' | '…' | '。' | '！' | '？'))
}

/// Groups segments into readable paragraphs: breaks at long pauses, and at sentence ends once a
/// paragraph is long enough.
pub fn paragraphs(segments: &[Segment]) -> Vec<Paragraph> {
    let mut out: Vec<Paragraph> = Vec::new();
    let mut chars = 0usize;
    for (i, s) in segments.iter().enumerate() {
        let start_new = match out.last() {
            None => true,
            Some(p) => {
                let prev = &segments[*p.segments.last().unwrap() as usize];
                let gap = s.start_ms.saturating_sub(prev.end_ms);
                gap >= PARAGRAPH_GAP_MS
                    || (chars >= PARAGRAPH_SOFT_CHARS && ends_sentence(&prev.text))
                    || chars >= PARAGRAPH_HARD_CHARS
            }
        };
        if start_new {
            out.push(Paragraph { start_ms: s.start_ms, end_ms: s.end_ms, segments: vec![i as u32] });
            chars = 0;
        } else {
            let p = out.last_mut().unwrap();
            p.end_ms = s.end_ms;
            p.segments.push(i as u32);
        }
        chars += s.text.len();
    }
    out
}

/// Splits `len` samples into processing blocks of about `target` samples, cutting at the
/// quietest 100 ms frame within `search` samples of each nominal boundary so blocks don't end
/// mid-word. `energy(start, end)` returns the mean energy of a range. Returns block starts
/// (first is 0); the last block runs to `len`.
pub fn block_starts(
    len: usize,
    target: usize,
    search: usize,
    frame: usize,
    energy: impl Fn(usize, usize) -> f32,
) -> Vec<usize> {
    let mut starts = vec![0usize];
    if target == 0 || frame == 0 {
        return starts;
    }
    let mut nominal = target;
    // Leave the last block at least half a block long.
    while nominal + target / 2 < len {
        let lo = nominal.saturating_sub(search).max(starts.last().unwrap() + frame);
        let hi = (nominal + search).min(len.saturating_sub(frame));
        let mut best = nominal.min(hi);
        let mut best_e = f32::MAX;
        let mut at = lo;
        while at + frame <= hi {
            let e = energy(at, at + frame);
            if e < best_e {
                best_e = e;
                best = at + frame / 2;
            }
            at += frame;
        }
        starts.push(best);
        nominal = best + target;
    }
    starts
}

pub fn fmt_clock(ms: u64) -> String {
    let s = ms / 1000;
    let (h, m, s) = (s / 3600, (s / 60) % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}

/// Parses `mm:ss`, `hh:mm:ss` or `[mm:ss]` (as produced by the generators) into ms.
pub fn parse_clock(s: &str) -> Option<u64> {
    let s = s.trim().trim_start_matches('[').trim_end_matches(']');
    // Ranges ("01:02 - 01:10") keep the start.
    let s = s.split(['-', '–']).next()?.trim();
    let parts: Vec<&str> = s.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut total = 0u64;
    for (i, p) in parts.iter().enumerate() {
        let v: u64 = p.trim().parse().ok()?;
        if i > 0 && v >= 60 {
            return None;
        }
        total = total * 60 + v;
    }
    Some(total * 1000)
}

fn fmt_stamp(ms: u64, sep: char) -> String {
    let (h, m, s, f) = (ms / 3_600_000, (ms / 60_000) % 60, (ms / 1000) % 60, ms % 1000);
    format!("{h:02}:{m:02}:{s:02}{sep}{f:03}")
}

/// Plain text, one paragraph per block, optionally with a `[mm:ss]` prefix.
pub fn to_txt(segments: &[Segment], timestamps: bool) -> String {
    let mut out = String::new();
    for p in paragraphs(segments) {
        if timestamps {
            out.push_str(&format!("[{}] ", fmt_clock(p.start_ms)));
        }
        out.push_str(&join(segments, &p));
        out.push_str("\n\n");
    }
    out.trim_end().to_string() + "\n"
}

fn join(segments: &[Segment], p: &Paragraph) -> String {
    p.segments
        .iter()
        .map(|&i| segments[i as usize].text.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Subtitles: long segments are split at word or clause boundaries so cues stay readable.
fn cues(segments: &[Segment]) -> Vec<(u64, u64, String)> {
    const MAX_CHARS: usize = 84;
    let mut out = Vec::new();
    for s in segments {
        let text = s.text.trim();
        if text.is_empty() {
            continue;
        }
        if text.chars().count() <= MAX_CHARS || s.end_ms <= s.start_ms {
            out.push((s.start_ms, s.end_ms.max(s.start_ms + 500), text.to_string()));
            continue;
        }
        // Split proportionally to character count at spaces.
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        for w in words {
            if !cur.is_empty() && cur.chars().count() + 1 + w.chars().count() > MAX_CHARS {
                lines.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        let total: usize = lines.iter().map(|l| l.chars().count()).sum::<usize>().max(1);
        let dur = s.end_ms - s.start_ms;
        let mut at = s.start_ms;
        for (i, l) in lines.iter().enumerate() {
            let end = if i + 1 == lines.len() {
                s.end_ms
            } else {
                at + dur * l.chars().count() as u64 / total as u64
            };
            out.push((at, end, l.clone()));
            at = end;
        }
    }
    out
}

pub fn to_srt(segments: &[Segment]) -> String {
    let mut out = String::new();
    for (n, (a, b, t)) in cues(segments).into_iter().enumerate() {
        out.push_str(&format!("{}\n{} --> {}\n{}\n\n", n + 1, fmt_stamp(a, ','), fmt_stamp(b, ','), t));
    }
    out
}

pub fn to_vtt(segments: &[Segment]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for (a, b, t) in cues(segments) {
        out.push_str(&format!("{} --> {}\n{}\n\n", fmt_stamp(a, '.'), fmt_stamp(b, '.'), t));
    }
    out
}

/// Markdown transcript with a title and timestamped paragraphs.
pub fn to_markdown(title: &str, segments: &[Segment]) -> String {
    let mut out = format!("# {title}\n\n");
    for p in paragraphs(segments) {
        out.push_str(&format!("**[{}]** {}\n\n", fmt_clock(p.start_ms), join(segments, &p)));
    }
    out
}

/// Compact transcript for language models: one `[mm:ss] text` line per paragraph.
pub fn to_prompt_lines(segments: &[Segment]) -> Vec<(u64, String)> {
    paragraphs(segments)
        .iter()
        .map(|p| (p.start_ms, format!("[{}] {}", fmt_clock(p.start_ms), join(segments, p))))
        .collect()
}

/// The segment that should be highlighted at `ms` (last one starting at or before it).
pub fn segment_at(segments: &[Segment], ms: u64) -> Option<usize> {
    let i = segments.partition_point(|s| s.start_ms <= ms);
    (i > 0).then(|| i - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn seg(idx: u32, a: u64, b: u64, t: &str) -> Segment {
        Segment { idx, start_ms: a, end_ms: b, text: t.into(), words: vec![], block: 0, edited: false }
    }

    #[test]
    fn paragraphs_break_at_pauses_and_sentence_ends() {
        let s = vec![
            seg(0, 0, 2000, "Bom dia a todos."),
            seg(1, 2100, 4000, "Vamos começar."),
            seg(2, 7000, 9000, "Primeiro ponto."),
        ];
        let p = paragraphs(&s);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].segments, vec![0, 1]);
        assert_eq!((p[1].start_ms, p[1].end_ms), (7000, 9000));
        // Long runs without pauses are split at a sentence end.
        let long: Vec<Segment> = (0..30)
            .map(|i| {
                seg(
                    i,
                    i as u64 * 1000,
                    i as u64 * 1000 + 900,
                    &"palavra ".repeat(6).trim_end().replace("palavra palavra", "palavra.").to_string(),
                )
            })
            .collect();
        assert!(paragraphs(&long).len() > 1);
    }

    #[test]
    fn clock_formats_and_parses() {
        assert_eq!(fmt_clock(65_000), "01:05");
        assert_eq!(fmt_clock(3_725_000), "1:02:05");
        assert_eq!(parse_clock("[01:05]"), Some(65_000));
        assert_eq!(parse_clock("1:02:05"), Some(3_725_000));
        assert_eq!(parse_clock("00:40 - 01:12"), Some(40_000));
        assert_eq!(parse_clock("12:75"), None);
        assert_eq!(parse_clock("ontem"), None);
    }

    #[test]
    fn srt_and_vtt_use_their_own_formats() {
        let s = vec![seg(0, 1_500, 3_250, " Olá."), seg(1, 3_400, 5_000, "Tudo bem?")];
        assert_eq!(
            to_srt(&s),
            "1\n00:00:01,500 --> 00:00:03,250\nOlá.\n\n2\n00:00:03,400 --> 00:00:05,000\nTudo bem?\n\n"
        );
        assert!(to_vtt(&s).starts_with("WEBVTT\n\n00:00:01.500 --> 00:00:03.250\nOlá.\n"));
    }

    #[test]
    fn long_segments_become_several_cues() {
        let text = "uma frase bem comprida ".repeat(10);
        let s = vec![seg(0, 0, 10_000, &text)];
        let c = cues(&s);
        assert!(c.len() >= 2);
        assert_eq!(c.first().unwrap().0, 0);
        assert_eq!(c.last().unwrap().1, 10_000);
        assert!(c.iter().all(|(_, _, t)| t.chars().count() <= 84));
        assert!(c.windows(2).all(|w| w[0].1 == w[1].0), "contiguous");
    }

    #[test]
    fn txt_and_markdown() {
        let s = vec![seg(0, 0, 1000, "A."), seg(1, 5000, 6000, "B.")];
        assert_eq!(to_txt(&s, true), "[00:00] A.\n\n[00:05] B.\n");
        assert_eq!(to_txt(&s, false), "A.\n\nB.\n");
        assert!(to_markdown("Reunião", &s).contains("**[00:05]** B."));
        assert_eq!(to_prompt_lines(&s)[1], (5000, "[00:05] B.".to_string()));
    }

    #[test]
    fn blocks_cut_at_the_quietest_point() {
        // 100 frames; quiet at frame 33 and 71.
        let energy = |a: usize, _b: usize| if a == 33 || a == 71 { 0.0 } else { 1.0 };
        let starts = block_starts(100, 30, 6, 1, energy);
        assert_eq!(starts[0], 0);
        assert_eq!(starts[1], 33);
        assert!(starts.windows(2).all(|w| w[1] > w[0]));
        assert_eq!(block_starts(10, 30, 6, 1, energy), vec![0]);
    }

    #[test]
    fn finds_segment_at_time() {
        let s = vec![seg(0, 0, 1000, "a"), seg(1, 2000, 3000, "b")];
        assert_eq!(segment_at(&s, 1500), Some(0));
        assert_eq!(segment_at(&s, 2000), Some(1));
        assert_eq!(segment_at(&[], 10), None);
    }
}
