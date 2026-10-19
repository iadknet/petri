//! Progress lines for long bench runs. They go to stderr only and never into a
//! report, so readings, summaries and determinism are unaffected.

use std::time::{Duration, Instant};

/// Gap between periodic lines inside a seed's tick loop.
pub(super) const INTERVAL: Duration = Duration::from_secs(30);

pub(super) struct Progress {
    started: Instant,
    last: Instant,
    interval: Duration,
}

impl Progress {
    pub(super) fn new(interval: Duration) -> Self {
        Self::starting_at(Instant::now(), interval)
    }

    fn starting_at(now: Instant, interval: Duration) -> Self {
        Self {
            started: now,
            last: now,
            interval,
        }
    }

    /// Print a line now.
    pub(super) fn log(&self, message: &str) {
        eprintln!("{}", self.line(Instant::now(), message));
    }

    /// Print the line `message` builds when an interval has passed since the
    /// last periodic line; `message` runs only then.
    pub(super) fn log_if_due(&mut self, message: impl FnOnce() -> String) {
        let now = Instant::now();
        if self.due(now) {
            eprintln!("{}", self.line(now, &message()));
        }
    }

    fn due(&mut self, now: Instant) -> bool {
        if now.duration_since(self.last) < self.interval {
            return false;
        }
        self.last = now;
        true
    }

    fn line(&self, now: Instant, message: &str) -> String {
        format!(
            "bench +{}s: {message}",
            now.duration_since(self.started).as_secs()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periodic_line_is_due_once_per_interval() {
        let start = Instant::now();
        let mut progress = Progress::starting_at(start, Duration::from_secs(30));
        assert!(!progress.due(start + Duration::from_secs(29)));
        assert!(progress.due(start + Duration::from_secs(30)));
        assert!(!progress.due(start + Duration::from_secs(59)));
        assert!(progress.due(start + Duration::from_secs(61)));
        assert!(!progress.due(start + Duration::from_secs(90)));
    }

    #[test]
    fn line_carries_elapsed_seconds_and_message() {
        let start = Instant::now();
        let progress = Progress::starting_at(start, INTERVAL);
        assert_eq!(
            progress.line(
                start + Duration::from_millis(75_900),
                "seed 11 tick 500/2000"
            ),
            "bench +75s: seed 11 tick 500/2000"
        );
    }
}
