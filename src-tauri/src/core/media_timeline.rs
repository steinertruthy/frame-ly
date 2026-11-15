//! Platform-neutral pieces of the recording pipeline, shared by native engines that are
//! written in Rust (Windows). Pure and unit-tested; the macOS engine has Swift equivalents
//! (`PauseTimeline.swift`, `AudioMixer.swift`) with the same behavior.
//!
//! Time is an integer tick count on one monotonic clock (on Windows: QPC in 100 ns units,
//! the clock Windows.Graphics.Capture and WASAPI both timestamp with).

// Used by the Windows engine (and the tests); macOS has its Swift equivalents.
#![cfg_attr(not(windows), allow(dead_code))]

/// Maps capture timestamps to output timestamps with paused intervals removed.
///
/// Pausing never stops capture (resuming is instant); samples that fall inside a pause are
/// dropped and every later sample shifts back by the total paused time. Works per sample, so
/// late buffers from before a pause are still placed correctly.
#[derive(Debug, Default, Clone)]
pub struct PauseTimeline {
    intervals: Vec<(i64, i64)>,
    paused_at: Option<i64>,
}

impl PauseTimeline {
    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    pub fn pause(&mut self, t: i64) {
        if self.paused_at.is_none() {
            self.paused_at = Some(t);
        }
    }

    pub fn resume(&mut self, t: i64) {
        if let Some(start) = self.paused_at.take() {
            if t > start {
                self.intervals.push((start, t));
            }
        }
    }

    /// Output time for `t`, or `None` if it was captured while paused.
    pub fn adjusted(&self, t: i64) -> Option<i64> {
        if matches!(self.paused_at, Some(p) if t >= p) {
            return None;
        }
        let mut shift = 0;
        for &(start, end) in &self.intervals {
            if t >= start && t < end {
                return None;
            }
            if t >= end {
                shift += end - start;
            }
        }
        Some(t - shift)
    }

    /// Total paused time up to `t`.
    pub fn paused_duration(&self, t: i64) -> i64 {
        let mut total: i64 = self.intervals.iter().map(|(s, e)| e - s).sum();
        if let Some(p) = self.paused_at {
            if t > p {
                total += t - p;
            }
        }
        total
    }
}

pub const SAMPLE_RATE: u32 = 48_000;
pub const CHANNELS: usize = 2;

#[derive(Debug, Clone)]
pub struct MixSource {
    pub gain: f32,
    written_until: Option<i64>,
    active: bool,
    /// Peak of the most recent buffer, after gain (0…1+).
    pub peak: f32,
    /// Diagnostics: frames received, and frames that arrived too late to be mixed.
    pub pushed_frames: i64,
    pub late_frames: i64,
    /// Highest peak received (before gain, so a muted source still shows whether its device
    /// delivered sound): tells a silent source from a working one.
    pub max_peak: f32,
}

/// Mixes timestamped PCM from several sources (system audio, microphone) into one continuous
/// 48 kHz interleaved stereo f32 stream on a shared timeline.
///
/// Each buffer is placed by its clock timestamp. Buffers landing within `snap` (10 ms) of
/// where the source's previous buffer ended are glued on contiguously (no clicks from jitter);
/// larger deviations (clock drift, gaps) resynchronize to the timestamp, so A/V offset stays
/// within ~10 ms for any length.
///
/// Output is released up to the point every active source has delivered, but never more than
/// `latency` behind the most advanced source, so a silent or stalled source never blocks the
/// others; its missing span is silence.
pub struct AudioMixer {
    pub sources: Vec<MixSource>,
    buffer: Vec<f32>,
    read_index: i64,
    origin: i64,
    ticks_per_second: i64,
    latency_frames: i64,
    snap_frames: i64,
}

impl AudioMixer {
    pub fn new(origin: i64, ticks_per_second: i64, gains: &[f32]) -> Self {
        Self {
            sources: gains
                .iter()
                .map(|&gain| MixSource {
                    gain,
                    written_until: None,
                    active: true,
                    peak: 0.0,
                    pushed_frames: 0,
                    late_frames: 0,
                    max_peak: 0.0,
                })
                .collect(),
            buffer: Vec::new(),
            read_index: 0,
            origin,
            ticks_per_second,
            latency_frames: (SAMPLE_RATE / 4) as i64,
            snap_frames: (SAMPLE_RATE / 100) as i64,
        }
    }

    pub fn frame_index(&self, t: i64) -> i64 {
        let ticks = (t - self.origin) as i128 * SAMPLE_RATE as i128;
        let tps = self.ticks_per_second as i128;
        ((ticks + tps / 2).div_euclid(tps)) as i64
    }

    pub fn time_of_frame(&self, frame: i64) -> i64 {
        self.origin + (frame as i128 * self.ticks_per_second as i128 / SAMPLE_RATE as i128) as i64
    }

    /// Adds interleaved stereo `samples` captured at clock time `t` from source `index`.
    pub fn push(&mut self, index: usize, t: i64, samples: &[f32]) {
        let frames = (samples.len() / CHANNELS) as i64;
        let Some(src) = self.sources.get(index) else { return };
        if frames == 0 || !src.active {
            return;
        }
        let mut start = self.frame_index(t);
        if let Some(w) = src.written_until {
            if (start - w).abs() <= self.snap_frames {
                start = w;
            }
        }
        let gain = src.gain;
        let src = &mut self.sources[index];
        let raw = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        src.peak = raw * gain;
        src.max_peak = src.max_peak.max(raw);
        src.written_until = Some(src.written_until.unwrap_or(0).max(start + frames));

        let skip = if start < self.read_index { (self.read_index - start).min(frames) } else { 0 };
        src.pushed_frames += frames;
        src.late_frames += skip;
        if skip >= frames {
            return;
        }
        let offset = ((start + skip - self.read_index) as usize) * CHANNELS;
        let count = ((frames - skip) as usize) * CHANNELS;
        if self.buffer.len() < offset + count {
            self.buffer.resize(offset + count, 0.0);
        }
        let from = skip as usize * CHANNELS;
        for i in 0..count {
            self.buffer[offset + i] += samples[from + i] * gain;
        }
    }

    /// Adds a source mid-recording (a device that appeared); returns its index.
    pub fn add_source(&mut self, gain: f32) -> usize {
        self.sources.push(MixSource {
            gain,
            written_until: None,
            active: true,
            peak: 0.0,
            pushed_frames: 0,
            late_frames: 0,
            max_peak: 0.0,
        });
        self.sources.len() - 1
    }

    /// A source that stopped for good (device unplugged): never wait for it again.
    pub fn deactivate(&mut self, index: usize) {
        if let Some(s) = self.sources.get_mut(index) {
            s.active = false;
            s.peak = 0.0;
        }
    }

    /// Releases mixed audio as `(start frame, samples)`. `flush` releases everything buffered.
    pub fn pull(&mut self, flush: bool) -> Option<(i64, Vec<f32>)> {
        let all_max = self.sources.iter().filter_map(|s| s.written_until).max()?;
        let active_min = self.sources.iter().filter(|s| s.active).filter_map(|s| s.written_until).min();
        let mut target =
            if flush { all_max } else { active_min.unwrap_or(all_max).max(all_max - self.latency_frames) };
        target = target.min(all_max);
        if target <= self.read_index {
            return None;
        }
        let count = (target - self.read_index) as usize * CHANNELS;
        if self.buffer.len() < count {
            self.buffer.resize(count, 0.0);
        }
        let mut out: Vec<f32> = self.buffer.drain(..count).collect();
        out.iter_mut().for_each(|x| *x = soft_clip(*x));
        let start = self.read_index;
        self.read_index = target;
        Some((start, out))
    }
}

/// Transparent below 0.8, smoothly limits above, never exceeds ±1.
pub fn soft_clip(x: f32) -> f32 {
    const T: f32 = 0.8;
    let a = x.abs();
    if a <= T {
        return x;
    }
    let limited = T + (1.0 - T) * ((a - T) / (1.0 - T)).tanh();
    limited.copysign(x)
}

/// Linear-interpolation resampler for interleaved stereo (devices not running at 48 kHz).
pub fn resample_stereo(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    let in_frames = input.len() / 2;
    if from == to || from == 0 || to == 0 || in_frames < 2 {
        return input.to_vec();
    }
    let out_frames = ((in_frames as f64) * to as f64 / from as f64).round() as usize;
    let step = from as f64 / to as f64;
    let mut out = vec![0.0; out_frames * 2];
    for f in 0..out_frames {
        let pos = f as f64 * step;
        let i = (pos as usize).min(in_frames - 2);
        let frac = (pos - i as f64) as f32;
        for c in 0..2 {
            out[f * 2 + c] = input[i * 2 + c] * (1.0 - frac) + input[(i + 1) * 2 + c] * frac;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TPS: i64 = 10_000_000; // 100 ns ticks

    #[test]
    fn timeline_drops_paused_spans_and_shifts_later_samples() {
        let mut t = PauseTimeline::default();
        assert_eq!(t.adjusted(100), Some(100));
        t.pause(1_000);
        assert!(t.is_paused());
        assert_eq!(t.adjusted(1_500), None);
        assert_eq!(t.adjusted(900), Some(900), "late buffer from before the pause");
        t.resume(3_000);
        assert_eq!(t.adjusted(2_000), None);
        assert_eq!(t.adjusted(3_500), Some(1_500));
        assert_eq!(t.paused_duration(4_000), 2_000);
        t.pause(5_000);
        assert_eq!(t.paused_duration(6_000), 3_000);
    }

    #[test]
    fn mixer_places_sources_on_one_timeline() {
        let mut m = AudioMixer::new(0, TPS, &[1.0, 0.5]);
        // 10 ms of 0.2 from source 0 at t=0; 10 ms of 0.4 from source 1 at t=0.
        m.push(0, 0, &vec![0.2; 960]);
        m.push(1, 0, &vec![0.4; 960]);
        let (start, out) = m.pull(false).unwrap();
        assert_eq!(start, 0);
        assert_eq!(out.len(), 960);
        assert!((out[0] - 0.4).abs() < 1e-6, "0.2 + 0.4 × 0.5");
        assert!((m.sources[1].peak - 0.2).abs() < 1e-6);
    }

    #[test]
    fn mixer_glues_jitter_and_does_not_wait_forever_for_a_silent_source() {
        let mut m = AudioMixer::new(0, TPS, &[1.0, 1.0]);
        m.push(0, 0, &vec![0.1; 960]); // frames 0..480
        m.push(1, 0, &vec![0.1; 960]); // source 1 delivers once, then stalls
                                       // Next buffer arrives 2 ms early (within the 10 ms snap): glued at 480.
        m.push(0, 100_000 - 20_000, &vec![0.1; 960]);
        assert_eq!(m.pull(false).unwrap().1.len() / 2, 480, "waits for source 1 while close");
        // Source 0 runs 0.4 s ahead: release stops 250 ms behind it, not at source 1.
        m.push(0, 4_000_000, &vec![0.1; 960]); // frame 19200..19680
        let (start, out) = m.pull(false).unwrap();
        assert_eq!((start, out.len() / 2), (480, (19_680 - 12_000 - 480) as usize));
        let (_, rest) = m.pull(true).unwrap();
        assert!(!rest.is_empty());
    }

    #[test]
    fn zero_gain_mutes_a_source_without_moving_the_timeline() {
        let mut m = AudioMixer::new(0, TPS, &[1.0, 1.0]);
        m.sources[1].gain = 0.0; // microphone muted
        m.push(0, 0, &vec![0.0; 960]);
        m.push(1, 0, &vec![0.5; 960]);
        let (start, out) = m.pull(false).unwrap();
        assert_eq!(start, 0);
        assert!(out.iter().all(|&x| x == 0.0));
        assert_eq!(m.sources[1].peak, 0.0, "meter shows silence");
    }

    #[test]
    fn sources_added_later_mix_in_and_report_their_peak() {
        let mut m = AudioMixer::new(0, TPS, &[1.0]);
        m.push(0, 0, &vec![0.0; 960]);
        let late = m.add_source(1.0);
        m.push(late, 0, &vec![0.25; 960]);
        let (_, out) = m.pull(true).unwrap();
        assert!((out[0] - 0.25).abs() < 1e-6);
        assert_eq!(m.sources[0].max_peak, 0.0, "silent output stays at zero");
        assert!((m.sources[late].max_peak - 0.25).abs() < 1e-6);
    }

    #[test]
    fn soft_clip_never_exceeds_one() {
        assert_eq!(soft_clip(0.5), 0.5);
        assert!(soft_clip(3.0) <= 1.0 && soft_clip(3.0) > 0.8);
        assert!(soft_clip(-3.0) >= -1.0);
    }

    #[test]
    fn resampler_keeps_duration() {
        let input = vec![0.0f32; 441 * 2];
        assert_eq!(resample_stereo(&input, 44_100, 48_000).len(), 480 * 2);
    }

    #[test]
    fn frame_time_round_trip() {
        let m = AudioMixer::new(1_000, TPS, &[1.0]);
        assert_eq!(m.frame_index(1_000 + TPS), 48_000);
        assert_eq!(m.time_of_frame(48_000), 1_000 + TPS);
    }
}
