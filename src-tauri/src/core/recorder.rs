//! Recording state machine and pause-aware clock. Pure (time is injected), fully tested.
//!
//! ```text
//! Idle ─start→ Countdown ─go→ Recording ⇄ Paused
//!                 │ cancel        │ stop      │ stop
//!                 ▼               ▼           ▼
//!               Idle          Finalizing ───→ Idle
//! ```

use super::errors::{AppError, AppResult};
use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Idle,
    Countdown,
    Recording,
    Paused,
    Finalizing,
}

#[derive(Debug)]
pub struct Recorder {
    phase: Phase,
    started: Option<Instant>,
    paused_at: Option<Instant>,
    paused_total: Duration,
}

impl Default for Recorder {
    fn default() -> Self {
        Self { phase: Phase::Idle, started: None, paused_at: None, paused_total: Duration::ZERO }
    }
}

fn invalid(from: Phase, action: &str) -> AppError {
    AppError::Runtime(format!("cannot {action} while {from:?}"))
}

impl Recorder {
    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_active(&self) -> bool {
        self.phase != Phase::Idle
    }

    pub fn begin_countdown(&mut self) -> AppResult<()> {
        match self.phase {
            Phase::Idle => {
                self.phase = Phase::Countdown;
                Ok(())
            }
            p => Err(invalid(p, "start")),
        }
    }

    pub fn started(&mut self, now: Instant) -> AppResult<()> {
        match self.phase {
            Phase::Idle | Phase::Countdown => {
                *self = Self { phase: Phase::Recording, started: Some(now), ..Default::default() };
                Ok(())
            }
            p => Err(invalid(p, "start")),
        }
    }

    pub fn pause(&mut self, now: Instant) -> AppResult<()> {
        match self.phase {
            Phase::Recording => {
                self.phase = Phase::Paused;
                self.paused_at = Some(now);
                Ok(())
            }
            p => Err(invalid(p, "pause")),
        }
    }

    pub fn resume(&mut self, now: Instant) -> AppResult<()> {
        match self.phase {
            Phase::Paused => {
                if let Some(p) = self.paused_at.take() {
                    self.paused_total += now.saturating_duration_since(p);
                }
                self.phase = Phase::Recording;
                Ok(())
            }
            p => Err(invalid(p, "resume")),
        }
    }

    pub fn finalizing(&mut self, now: Instant) -> AppResult<()> {
        match self.phase {
            Phase::Recording | Phase::Paused => {
                if self.phase == Phase::Paused {
                    self.resume(now)?;
                }
                self.phase = Phase::Finalizing;
                Ok(())
            }
            p => Err(invalid(p, "stop")),
        }
    }

    /// Back to idle from any phase (finished, cancelled countdown, or failure).
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Recorded time so far, excluding pauses.
    pub fn elapsed(&self, now: Instant) -> Duration {
        let Some(start) = self.started else { return Duration::ZERO };
        let mut paused = self.paused_total;
        if let Some(p) = self.paused_at {
            paused += now.saturating_duration_since(p);
        }
        now.saturating_duration_since(start).saturating_sub(paused)
    }
}

/// `mm:ss` (or `h:mm:ss` past an hour) for the tray title and UI timer.
pub fn format_elapsed(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_lifecycle_with_pauses() {
        let t0 = Instant::now();
        let at = |s: u64| t0 + Duration::from_secs(s);
        let mut r = Recorder::default();
        r.begin_countdown().unwrap();
        assert_eq!(r.phase(), Phase::Countdown);
        r.started(at(3)).unwrap();
        r.pause(at(13)).unwrap();
        assert_eq!(r.elapsed(at(20)), Duration::from_secs(10), "frozen while paused");
        r.resume(at(20)).unwrap();
        assert_eq!(r.elapsed(at(25)), Duration::from_secs(15));
        r.pause(at(25)).unwrap();
        r.finalizing(at(30)).unwrap();
        assert_eq!(r.phase(), Phase::Finalizing);
        assert_eq!(r.elapsed(at(30)), Duration::from_secs(15));
        r.reset();
        assert!(!r.is_active());
    }

    #[test]
    fn rejects_invalid_transitions() {
        let now = Instant::now();
        let mut r = Recorder::default();
        assert!(r.pause(now).is_err());
        assert!(r.resume(now).is_err());
        assert!(r.finalizing(now).is_err());
        r.started(now).unwrap();
        assert!(r.begin_countdown().is_err());
        assert!(r.started(now).is_err());
        assert!(r.resume(now).is_err());
    }

    #[test]
    fn formats_elapsed() {
        assert_eq!(format_elapsed(Duration::from_secs(7)), "00:07");
        assert_eq!(format_elapsed(Duration::from_secs(754)), "12:34");
        assert_eq!(format_elapsed(Duration::from_secs(3723)), "1:02:03");
    }
}
