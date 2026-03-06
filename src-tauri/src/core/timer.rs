//! A pure, unit-testable countdown timer.
//! Takes an explicit `Instant` per query instead of reading the clock itself.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct SessionTimer {
    started_at: Instant,
    duration: Duration,
}

impl SessionTimer {
    pub fn new(started_at: Instant, duration: Duration) -> Self {
        Self {
            started_at,
            duration,
        }
    }

    /// Time remaining at `now`. Saturates at zero; never panics or goes negative.
    pub fn remaining(&self, now: Instant) -> Duration {
        let elapsed = now.saturating_duration_since(self.started_at);
        self.duration.saturating_sub(elapsed)
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        now.saturating_duration_since(self.started_at)
    }

    pub fn is_expired(&self, now: Instant) -> bool {
        self.remaining(now) == Duration::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_counts_down() {
        let start = Instant::now();
        let timer = SessionTimer::new(start, Duration::from_secs(60));

        assert_eq!(timer.remaining(start), Duration::from_secs(60));
        assert_eq!(
            timer.remaining(start + Duration::from_secs(10)),
            Duration::from_secs(50)
        );
    }

    #[test]
    fn remaining_saturates_at_zero_past_expiry() {
        let start = Instant::now();
        let timer = SessionTimer::new(start, Duration::from_secs(10));

        assert_eq!(
            timer.remaining(start + Duration::from_secs(999)),
            Duration::ZERO
        );
        assert!(timer.is_expired(start + Duration::from_secs(999)));
    }

    #[test]
    fn is_expired_exactly_at_duration() {
        let start = Instant::now();
        let timer = SessionTimer::new(start, Duration::from_secs(5));
        assert!(timer.is_expired(start + Duration::from_secs(5)));
        assert!(!timer.is_expired(start + Duration::from_millis(4999)));
    }
}
