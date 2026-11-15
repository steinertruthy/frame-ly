//! Streaming conversion of decoded audio (any rate, any channel count, interleaved f32) to the
//! 16 kHz mono stream speech recognition expects. Windowed-sinc (Blackman) low-pass resampler;
//! feed chunks of any size, then `finish`. Used on Windows (macOS decodes straight to 16 kHz).

use std::f64::consts::PI;

pub const ASR_RATE: u32 = 16_000;
/// Zero crossings on each side of the kernel.
const ZEROS: usize = 16;

pub struct Resampler {
    in_rate: u32,
    out_rate: u32,
    channels: usize,
    /// Mono input not yet fully consumed; `pos` is relative to `buf[0]`.
    buf: Vec<f32>,
    pos: f64,
    step: f64,
    cutoff: f64,
    half: usize,
    frames_in: u64,
    frames_out: u64,
}

impl Resampler {
    pub fn new(in_rate: u32, channels: u16, out_rate: u32) -> Self {
        let step = in_rate as f64 / out_rate as f64;
        // Low-pass at 95 % of the lower Nyquist frequency, relative to the input rate.
        let cutoff = 0.5 * (out_rate as f64 / in_rate as f64).min(1.0) * 0.95;
        let half = (ZEROS as f64 / (2.0 * cutoff)).ceil() as usize;
        // Start with `half` samples of silence so the first outputs have full history.
        Self {
            in_rate,
            out_rate,
            channels: channels.max(1) as usize,
            buf: vec![0.0; half],
            pos: half as f64,
            step,
            cutoff,
            half,
            frames_in: 0,
            frames_out: 0,
        }
    }

    fn kernel(&self, x: f64) -> f64 {
        let span = self.half as f64;
        if x.abs() >= span {
            return 0.0;
        }
        let sinc =
            if x.abs() < 1e-9 { 2.0 * self.cutoff } else { (2.0 * PI * self.cutoff * x).sin() / (PI * x) };
        let w = 0.42 + 0.5 * (PI * x / span).cos() + 0.08 * (2.0 * PI * x / span).cos();
        sinc * w
    }

    fn emit(&mut self, out: &mut Vec<f32>, flush: bool) {
        let passthrough = self.in_rate == self.out_rate;
        loop {
            let center = self.pos;
            let needed = center.floor() as usize + self.half + 1;
            if needed > self.buf.len() && !flush {
                break;
            }
            if center >= self.buf.len() as f64 {
                break;
            }
            let v = if passthrough {
                self.buf[center as usize] as f64
            } else {
                let lo = (center.floor() as isize - self.half as isize + 1).max(0) as usize;
                let hi = (center.floor() as usize + self.half).min(self.buf.len() - 1);
                (lo..=hi).map(|i| self.buf[i] as f64 * self.kernel(i as f64 - center)).sum::<f64>()
            };
            out.push(v.clamp(-1.0, 1.0) as f32);
            self.frames_out += 1;
            self.pos += self.step;
        }
        // Drop history we no longer need, keeping `half` samples before the next center.
        let keep_from = (self.pos.floor() as usize).saturating_sub(self.half);
        if keep_from > 0 && keep_from <= self.buf.len() {
            self.buf.drain(..keep_from);
            self.pos -= keep_from as f64;
        }
    }

    /// Adds interleaved samples; appends the converted output to `out`.
    pub fn push(&mut self, interleaved: &[f32], out: &mut Vec<f32>) {
        let ch = self.channels;
        let before = self.buf.len();
        self.buf.extend(interleaved.chunks_exact(ch).map(|f| f.iter().sum::<f32>() / ch as f32));
        self.frames_in += (self.buf.len() - before) as u64;
        self.emit(out, false);
    }

    /// Flushes the tail: the output ends up exactly `ceil(frames_in / step)` samples long.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        let total = (self.frames_in as f64 / self.step).ceil() as u64;
        self.buf.extend(std::iter::repeat(0.0).take(self.half + 1));
        let mut tail = Vec::new();
        self.emit(&mut tail, true);
        let want = total.saturating_sub(self.frames_out - tail.len() as u64) as usize;
        tail.truncate(want);
        out.extend_from_slice(&tail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(rate: u32, hz: f64, secs: f64, channels: usize) -> Vec<f32> {
        let n = (rate as f64 * secs) as usize;
        (0..n)
            .flat_map(|i| {
                std::iter::repeat((2.0 * PI * hz * i as f64 / rate as f64).sin() as f32 * 0.5).take(channels)
            })
            .collect()
    }

    fn convert(input: &[f32], rate: u32, ch: u16, chunk: usize) -> Vec<f32> {
        let mut r = Resampler::new(rate, ch, ASR_RATE);
        let mut out = Vec::new();
        for c in input.chunks(chunk * ch as usize) {
            r.push(c, &mut out);
        }
        r.finish(&mut out);
        out
    }

    /// Amplitude of `hz` in `x` (16 kHz) by correlation.
    fn amplitude(x: &[f32], hz: f64) -> f64 {
        let (mut s, mut c) = (0.0, 0.0);
        for (i, v) in x.iter().enumerate() {
            let t = 2.0 * PI * hz * i as f64 / ASR_RATE as f64;
            s += *v as f64 * t.sin();
            c += *v as f64 * t.cos();
        }
        2.0 * (s * s + c * c).sqrt() / x.len() as f64
    }

    #[test]
    fn keeps_duration_and_speech_band() {
        let out = convert(&tone(48_000, 1_000.0, 2.0, 2), 48_000, 2, 1_000);
        assert!((out.len() as i64 - 32_000).abs() <= 2, "len {}", out.len());
        let a = amplitude(&out[2000..30_000], 1_000.0);
        assert!((a - 0.5).abs() < 0.02, "1 kHz preserved, got {a}");
    }

    #[test]
    fn removes_content_above_nyquist() {
        // 12 kHz would alias to 4 kHz at 16 kHz without filtering.
        let out = convert(&tone(44_100, 12_000.0, 1.0, 1), 44_100, 1, 4_096);
        assert!(amplitude(&out[2000..14_000], 4_000.0) < 0.01);
        assert!(out.iter().all(|v| v.abs() <= 1.0));
    }

    #[test]
    fn chunk_size_does_not_change_the_result() {
        let input = tone(22_050, 440.0, 0.5, 1);
        let a = convert(&input, 22_050, 1, 1);
        let b = convert(&input, 22_050, 1, 10_000);
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-6));
    }

    #[test]
    fn same_rate_passes_through() {
        let input = tone(16_000, 300.0, 0.25, 1);
        let out = convert(&input, 16_000, 1, 333);
        assert_eq!(out.len(), input.len());
        assert!(out.iter().zip(&input).all(|(a, b)| (a - b).abs() < 1e-6));
    }
}
