//! Inactivity tracking that ignores idle resets caused by our own synthetic input.
//! Input is attributed by timestamp: a reset inside our last action's window is ours.

use std::time::{Duration, Instant};

/// Slack for OS timestamp granularity (Windows ticks are ~16ms).
const SLACK_BEFORE: Duration = Duration::from_millis(50);
const SLACK_AFTER: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy)]
pub struct InactivityMonitor {
    threshold: Duration,
    last_real_input_at: Option<Instant>,
    last_synthetic: Option<(Instant, Instant)>,
    effective_idle: Duration,
}

impl InactivityMonitor {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            last_real_input_at: None,
            last_synthetic: None,
            effective_idle: Duration::ZERO,
        }
    }

    /// Records the span during which we injected input.
    pub fn record_synthetic_action(&mut self, started: Instant, ended: Instant) {
        self.last_synthetic = Some((started, ended));
    }

    /// Treats an unreadable idle time as the user being present, so automation stops.
    pub fn mark_unknown(&mut self, now: Instant) {
        self.last_real_input_at = Some(now);
        self.effective_idle = Duration::ZERO;
    }

    /// Feeds an OS idle reading taken at `now`; returns idle time since the last real input.
    pub fn poll(&mut self, now: Instant, os_idle: Duration) -> Duration {
        let Some(last_input_at) = now.checked_sub(os_idle) else {
            self.last_real_input_at = None;
            self.effective_idle = os_idle;
            return os_idle;
        };
        if !self.is_synthetic(last_input_at) {
            self.last_real_input_at = Some(last_input_at);
        }
        self.effective_idle = match self.last_real_input_at {
            Some(real) => now.saturating_duration_since(real),
            None => os_idle,
        };
        self.effective_idle
    }

    fn is_synthetic(&self, input_at: Instant) -> bool {
        self.last_synthetic.is_some_and(|(started, ended)| {
            input_at + SLACK_BEFORE >= started && input_at <= ended + SLACK_AFTER
        })
    }

    pub fn effective_idle(&self) -> Duration {
        self.effective_idle
    }

    pub fn is_inactive(&self) -> bool {
        self.effective_idle >= self.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: Duration = Duration::from_secs(300);

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn becomes_inactive_once_threshold_reached() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);

        monitor.poll(t0, secs(299));
        assert!(!monitor.is_inactive());
        monitor.poll(t0 + secs(1), secs(300));
        assert!(monitor.is_inactive());
    }

    #[test]
    fn real_input_resets_idle() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);

        monitor.poll(t0, secs(400));
        monitor.poll(t0 + secs(1), secs(0));
        assert!(!monitor.is_inactive());
    }

    #[test]
    fn synthetic_input_never_looks_like_the_user_returning() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);
        monitor.poll(t0, secs(400));

        let action_at = t0 + secs(1);
        monitor.record_synthetic_action(action_at, action_at + Duration::from_millis(200));

        // Idle keeps counting from our action.
        for later in [2, 3, 10, 60] {
            monitor.poll(action_at + secs(later), secs(later));
            assert!(monitor.is_inactive(), "{later}s after our action");
        }
        assert_eq!(monitor.effective_idle(), secs(461));
    }

    #[test]
    fn stays_inactive_across_an_hour_of_synthetic_actions() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);
        let mut last_input = t0 - secs(300);
        monitor.poll(t0, t0 - last_input);

        for tick in 1..=3600 {
            let now = t0 + secs(tick);
            monitor.poll(now, now - last_input);
            assert!(monitor.is_inactive(), "tick {tick}");
            if tick % 10 == 0 {
                monitor.record_synthetic_action(now, now);
                last_input = now;
            }
        }
    }

    #[test]
    fn real_input_half_a_second_after_our_action_is_caught() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);
        monitor.poll(t0, secs(400));
        let ended = t0 + Duration::from_millis(300);
        monitor.record_synthetic_action(t0, ended);

        let typed_at = ended + Duration::from_millis(500);
        let now = t0 + secs(1);
        monitor.poll(now, now - typed_at);
        assert!(!monitor.is_inactive());
    }

    #[test]
    fn an_unreadable_idle_time_counts_as_present_until_a_reading_returns() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);
        monitor.poll(t0, secs(400));
        monitor.mark_unknown(t0 + secs(1));
        assert!(!monitor.is_inactive());

        monitor.poll(t0 + secs(2), secs(402));
        assert!(monitor.is_inactive());
    }

    #[test]
    fn real_input_after_a_synthetic_action_is_caught() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let t0 = Instant::now() + secs(1000);
        monitor.poll(t0, secs(400));
        monitor.record_synthetic_action(t0 + secs(1), t0 + secs(1));

        // Real input at t0+10.
        monitor.poll(t0 + secs(10), secs(0));
        assert!(!monitor.is_inactive());
    }

    #[test]
    fn idle_longer_than_the_monotonic_clock_is_taken_as_is() {
        let mut monitor = InactivityMonitor::new(THRESHOLD);
        let now = Instant::now();
        let huge = secs(u64::MAX / 4);
        assert_eq!(monitor.poll(now, huge), huge);
        assert!(monitor.is_inactive());
    }
}
