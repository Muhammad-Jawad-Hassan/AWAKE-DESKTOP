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

    pub fn extend(&mut self, extra: Duration) {
        self.duration += extra;
    }
}

/// Rejects zero-length sessions and sessions longer than 24 hours.
pub fn validate_duration(duration: Duration) -> Result<(), DurationError> {
    const MAX_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

    if duration.is_zero() {
        Err(DurationError::TooShort)
    } else if duration > MAX_DURATION {
        Err(DurationError::TooLong { max: MAX_DURATION })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum DurationError {
    #[error("session duration must be greater than zero")]
    TooShort,
    #[error("session duration must not exceed {max:?}")]
    TooLong { max: Duration },
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

    #[test]
    fn rejects_zero_duration() {
        assert_eq!(
            validate_duration(Duration::ZERO),
            Err(DurationError::TooShort)
        );
    }

    #[test]
    fn rejects_excessive_duration() {
        let too_long = Duration::from_secs(25 * 60 * 60);
        assert!(validate_duration(too_long).is_err());
    }

    #[test]
    fn accepts_reasonable_duration() {
        assert!(validate_duration(Duration::from_secs(6 * 60 * 60)).is_ok());
    }

    #[test]
    fn extend_pushes_remaining_time_out() {
        let start = Instant::now();
        let mut timer = SessionTimer::new(start, Duration::from_secs(10));
        timer.extend(Duration::from_secs(20));
        assert_eq!(timer.remaining(start), Duration::from_secs(30));
    }
}
