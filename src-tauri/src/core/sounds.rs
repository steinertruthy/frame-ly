//! FrameLy's feedback sounds, synthesized at runtime (no audio assets to ship or license).
//!
//! Each cue is a couple of soft, bell-like notes: a sine with gentle upper partials, a fast
//! attack and an exponential decay. Rendered once into a 16-bit mono WAV that the platform
//! layer plays from memory.

use std::sync::OnceLock;

const RATE: u32 = 48_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    /// A screenshot was taken: a light, quick "tick-tick".
    Shutter,
    /// Recording began: two rising notes.
    RecordStart,
    /// Recording ended: the same two notes, falling.
    RecordStop,
}

/// (start s, frequency Hz, length s, amplitude)
type Note = (f32, f32, f32, f32);

const E5: f32 = 659.25;
const B5: f32 = 987.77;

fn notes(cue: Cue) -> &'static [Note] {
    match cue {
        Cue::Shutter => &[(0.0, 1567.98, 0.10, 0.20), (0.04, 2093.0, 0.14, 0.15)],
        Cue::RecordStart => &[(0.0, E5, 0.38, 0.22), (0.10, B5, 0.50, 0.20)],
        Cue::RecordStop => &[(0.0, B5, 0.38, 0.20), (0.10, E5, 0.55, 0.22)],
    }
}

fn render(cue: Cue) -> Vec<i16> {
    let notes = notes(cue);
    let end = notes.iter().map(|n| n.0 + n.2).fold(0.0f32, f32::max) + 0.02;
    let mut buf = vec![0f32; (end * RATE as f32) as usize];
    for &(start, freq, len, amp) in notes {
        let first = (start * RATE as f32) as usize;
        let count = (len * RATE as f32) as usize;
        let attack = 0.004 * RATE as f32;
        let tau = len / 5.0;
        for i in 0..count {
            let t = i as f32 / RATE as f32;
            let env = (i as f32 / attack).min(1.0) * (-t / tau).exp();
            let w = std::f32::consts::TAU * freq * t;
            let s = w.sin() + 0.15 * (2.0 * w).sin() + 0.05 * (3.0 * w).sin();
            if let Some(out) = buf.get_mut(first + i) {
                *out += amp * env * s;
            }
        }
    }
    // Short fade-out so the file never ends on a click.
    let fade = (0.01 * RATE as f32) as usize;
    let n = buf.len();
    for (i, s) in buf.iter_mut().skip(n - fade).enumerate() {
        *s *= 1.0 - i as f32 / fade as f32;
    }
    buf.iter().map(|s| (s.tanh() * i16::MAX as f32) as i16).collect()
}

fn wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes()); // byte rate
    out.extend_from_slice(&2u16.to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

/// The cue as a complete WAV file, rendered on first use and kept for the process lifetime
/// (Windows plays it asynchronously straight from this memory).
pub fn wav_bytes(cue: Cue) -> &'static [u8] {
    static CACHE: [OnceLock<Vec<u8>>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let i = match cue {
        Cue::Shutter => 0,
        Cue::RecordStart => 1,
        Cue::RecordStop => 2,
    };
    CACHE[i].get_or_init(|| wav(&render(cue)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_valid_pcm() {
        let w = wav_bytes(Cue::Shutter);
        assert_eq!(&w[0..4], b"RIFF");
        assert_eq!(&w[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(w[4..8].try_into().unwrap()) as usize, w.len() - 8);
        assert_eq!(u32::from_le_bytes(w[24..28].try_into().unwrap()), RATE);
        assert_eq!(u32::from_le_bytes(w[40..44].try_into().unwrap()) as usize, w.len() - 44);
    }

    #[test]
    fn cues_are_short_soft_and_audible() {
        for cue in [Cue::Shutter, Cue::RecordStart, Cue::RecordStop] {
            let s = render(cue);
            let secs = s.len() as f32 / RATE as f32;
            assert!(secs > 0.1 && secs < 0.8, "{cue:?} lasts {secs}s");
            let peak = s.iter().map(|v| v.unsigned_abs()).max().unwrap();
            assert!(peak > 3_000, "{cue:?} is too quiet");
            assert!(peak < 20_000, "{cue:?} is too loud ({peak})");
            assert!(s.last().unwrap().unsigned_abs() < 50, "{cue:?} ends on a click");
        }
    }

    /// `FRAMELY_DUMP_SOUNDS=<dir> cargo test sounds` writes the cues out for listening.
    #[test]
    fn dump_for_listening() {
        let Ok(dir) = std::env::var("FRAMELY_DUMP_SOUNDS") else { return };
        for (cue, name) in
            [(Cue::Shutter, "shutter"), (Cue::RecordStart, "record-start"), (Cue::RecordStop, "record-stop")]
        {
            std::fs::write(std::path::Path::new(&dir).join(format!("{name}.wav")), wav_bytes(cue)).unwrap();
        }
    }

    #[test]
    fn start_rises_and_stop_falls() {
        let first = |c| notes(c)[0].1;
        let last = |c: Cue| notes(c).last().unwrap().1;
        assert!(first(Cue::RecordStart) < last(Cue::RecordStart));
        assert!(first(Cue::RecordStop) > last(Cue::RecordStop));
    }
}
