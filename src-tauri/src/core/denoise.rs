//! Microphone noise suppression (RNNoise, via the pure-Rust `nnnoiseless`), shared by both
//! recording engines: Windows calls it from its audio thread, macOS from Swift through a
//! callback registered at startup.
//!
//! `process` works in place on interleaved stereo f32 at 48 kHz and always returns as many
//! samples as it receives, so callers keep their buffers and timestamps; the audio comes out
//! `LATENCY_FRAMES` later, which callers subtract from the buffer's timestamp to stay in sync.

use nnnoiseless::DenoiseState;
use std::collections::VecDeque;

const FRAME: usize = DenoiseState::FRAME_SIZE; // 480 samples = 10 ms at 48 kHz
/// Fixed delay between a sample going in and its denoised version coming out: one frame of
/// buffering plus RNNoise's own frame of overlap (measured in the tests).
pub const LATENCY_FRAMES: usize = 2 * FRAME;

pub struct Denoiser {
    state: Box<DenoiseState<'static>>,
    /// Mono input waiting to fill a frame, and denoised mono output waiting to be returned.
    pending: Vec<f32>,
    ready: VecDeque<f32>,
}

impl Default for Denoiser {
    fn default() -> Self {
        Self::new()
    }
}

impl Denoiser {
    pub fn new() -> Self {
        // One frame of silence ahead of the output makes the stream length-preserving.
        Self {
            state: DenoiseState::new(),
            pending: Vec::with_capacity(FRAME),
            ready: vec![0.0; FRAME].into(),
        }
    }

    /// Denoises interleaved stereo in place (the microphone is mono: both channels get the
    /// denoised mix of the two).
    pub fn process(&mut self, stereo: &mut [f32]) {
        let mut out = [0.0f32; FRAME];
        for pair in stereo.chunks_exact_mut(2) {
            // RNNoise works on 16-bit-range floats.
            self.pending.push((pair[0] + pair[1]) * 0.5 * 32_768.0);
            if self.pending.len() == FRAME {
                self.state.process_frame(&mut out, &self.pending);
                self.ready.extend(out.iter().map(|s| s / 32_768.0));
                self.pending.clear();
            }
            let s = self.ready.pop_front().unwrap_or(0.0);
            pair[0] = s;
            pair[1] = s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(n: usize, seed: &mut u32) -> Vec<f32> {
        (0..n)
            .map(|_| {
                *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (*seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5
            })
            .collect()
    }

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|s| s * s).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }

    #[test]
    fn keeps_length_in_any_chunking() {
        let mut d = Denoiser::new();
        for n in [1, 7, 480, 961, 3] {
            let mut buf = vec![0.1f32; n * 2];
            d.process(&mut buf);
            assert_eq!(buf.len(), n * 2);
        }
    }

    #[test]
    fn removes_steady_background_noise() {
        let mut seed = 7;
        let mut d = Denoiser::new();
        // 3 s of hiss at a level a noisy room or fan gives a laptop mic.
        let mono: Vec<f32> = noise(48_000 * 3, &mut seed).iter().map(|s| s * 0.06).collect();
        let mut stereo: Vec<f32> = mono.iter().flat_map(|&s| [s, s]).collect();
        let before = rms(&stereo[48_000 * 4..]);
        d.process(&mut stereo);
        let after = rms(&stereo[48_000 * 4..]); // after the model has adapted
        eprintln!("hiss rms {before:.4} → {after:.4}");
        assert!(after < before * 0.3, "hiss {before:.4} → {after:.4}");
    }

    #[test]
    fn declared_latency_matches_the_signal_delay() {
        // A burst after silence: its onset must come out LATENCY_FRAMES later (±2 ms).
        let mut d = Denoiser::new();
        let onset = 48_000;
        let mut mono = vec![0.0f32; 48_000 * 2];
        for (i, s) in mono[onset..].iter_mut().enumerate() {
            *s = 0.5 * (2.0 * std::f32::consts::PI * 300.0 * i as f32 / 48_000.0).sin();
        }
        let mut stereo: Vec<f32> = mono.iter().flat_map(|&s| [s, s]).collect();
        d.process(&mut stereo);
        let found = stereo.chunks_exact(2).position(|p| p[0].abs() > 0.02).expect("signal kept");
        let delay = found as i64 - onset as i64;
        eprintln!("measured delay {delay} frames, declared {LATENCY_FRAMES}");
        assert!(
            (delay - LATENCY_FRAMES as i64).abs() <= 96,
            "delay {delay} frames, declared {LATENCY_FRAMES}"
        );
    }
}
