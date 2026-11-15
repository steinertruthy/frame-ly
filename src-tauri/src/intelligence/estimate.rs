//! Time estimates before starting. Based on this computer's own previous runs (median of the
//! last ten with the same model and acceleration) when there are any; otherwise on reference
//! rates measured on representative machines, and then marked approximate.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimeEstimate {
    pub ms: u64,
    /// No history on this computer yet: reference rates.
    pub approximate: bool,
    /// Runs the estimate is based on.
    pub runs: usize,
}

/// Reference ms of work per second of audio (speech recognition).
fn speech_reference(model_id: &str, accel: &str) -> f64 {
    let metal = accel == "metal";
    match model_id {
        // Measured on an M3 Pro (Metal): small 6.1 s and large-v3-turbo 12.9 s for 217 s of audio.
        "speech-fast" => if metal { 30.0 } else { 250.0 },
        "speech-max" => if metal { 350.0 } else { 3_000.0 },
        _ => if metal { 60.0 } else { 800.0 },
    }
}

fn load_reference(model_id: &str) -> f64 {
    match model_id {
        "speech-fast" => 1_500.0,
        "speech-max" => 8_000.0,
        "language-basic" | "language-balanced" => 6_000.0,
        "language-advanced" => 10_000.0,
        _ => 4_000.0,
    }
}

pub fn transcription(duration_ms: u64, model_id: &str, accel: &str, history: Option<(f64, usize)>) -> TimeEstimate {
    let secs = duration_ms as f64 / 1000.0;
    // Decoding the audio is ~1 % of real time with the native decoders.
    let decode = secs * 10.0;
    match history {
        Some((rate, runs)) => TimeEstimate { ms: (secs * rate + decode + load_reference(model_id) * 0.5) as u64, approximate: false, runs },
        None => TimeEstimate { ms: (secs * speech_reference(model_id, accel) + decode + load_reference(model_id)) as u64, approximate: true, runs: 0 },
    }
}

/// Reference ms per transcript character for summaries/topics (local model).
fn generation_reference(task: &str, model_id: &str, accel: &str) -> f64 {
    let base = match (task, model_id) {
        ("summary", "language-basic") => 4.0,
        ("summary", "language-advanced") => 12.0,
        ("summary", _) => 7.0,
        (_, "language-basic") => 10.0,
        (_, "language-advanced") => 35.0,
        _ => 20.0,
    };
    if accel == "metal" {
        base
    } else {
        base * 4.0
    }
}

pub fn generation(chars: usize, task: &str, model_id: &str, accel: &str, history: Option<(f64, usize)>) -> TimeEstimate {
    match history {
        Some((rate, runs)) => TimeEstimate { ms: (chars as f64 * rate) as u64, approximate: false, runs },
        None => TimeEstimate {
            ms: (chars as f64 * generation_reference(task, model_id, accel) + load_reference(model_id)) as u64,
            approximate: true,
            runs: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_history_when_available() {
        let approx = transcription(600_000, "speech-balanced", "metal", None);
        assert!(approx.approximate && approx.runs == 0);
        // 10 min at the measured 60 ms/s ≈ 36 s + decode + load.
        assert!((36_000..50_000).contains(&approx.ms), "{}", approx.ms);
        let real = transcription(600_000, "speech-balanced", "metal", Some((100.0, 4)));
        assert!(!real.approximate && real.runs == 4 && real.ms > approx.ms);
        assert!(transcription(600_000, "speech-balanced", "cpu", None).ms > approx.ms * 5);
        assert!(transcription(600_000, "speech-max", "metal", None).ms > approx.ms);
    }

    #[test]
    fn generation_scales_with_length_and_model() {
        let a = generation(10_000, "summary", "language-balanced", "metal", None);
        let b = generation(20_000, "summary", "language-balanced", "metal", None);
        assert!(b.ms > a.ms && a.approximate);
        assert!(generation(10_000, "topics", "language-balanced", "metal", None).ms > a.ms);
        assert_eq!(generation(1_000, "summary", "x", "metal", Some((2.0, 3))).ms, 2_000);
    }
}
