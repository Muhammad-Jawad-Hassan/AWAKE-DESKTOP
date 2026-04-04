//! Inactivity tracking that filters out idle-time resets caused by our own
//! synthetic input, not just the real user. See docs/architecture.md.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityState {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Copy)]
pub struct InactivityMonitor {
    threshold: Duration,
    /// How long after a synthetic action we distrust the raw OS idle reading.
    grace_period: Duration,
    effective_idle: Duration,
    last_synthetic_action_at: Option<Instant>,
    last_poll_at: Option<Instant>,
}

/// Default grace window after a synthetic action before we trust OS idle readings again.
pub const DEFAULT_GRACE_PERIOD: Duration = Duration::from_secs(2);

impl InactivityMonitor {
    pub fn new(threshold: Duration) -> Self {
        Self::with_grace_period(threshold, DEFAULT_GRACE_PERIOD)
    }

    pub fn with_grace_period(threshold: Duration, grace_period: Duration) -> Self {
        Self {
            threshold,
            grace_period,
            effective_idle: Duration::ZERO,
            last_synthetic_action_at: None,
            last_poll_at: None,
        }
    }

    /// Call right after the engine performs a synthetic mouse/keyboard action.
    pub fn notify_synthetic_action(&mut self, now: Instant) {
        self.last_synthetic_action_at = Some(now);
    }

    /// Feeds a fresh OS idle reading; returns the filtered effective idle time.
    pub fn poll(&mut self, now: Instant, os_idle: Duration) -> Duration {
        let elapsed_since_last_poll = match self.last_poll_at {
            Some(last) => now.saturating_duration_since(last),
            None => Duration::ZERO,
        };
        self.last_poll_at = Some(now);

        let in_grace_window = self
            .last_synthetic_action_at
            .is_some_and(|at| now.saturating_duration_since(at) < self.grace_period);

        if in_grace_window {
            // Ignore the OS reading; keep counting as if nothing happened.
            self.effective_idle += elapsed_since_last_poll;
        } else {
            self.effective_idle = os_idle;
        }

        self.effective_idle
    }

    pub fn effective_idle(&self) -> Duration {
        self.effective_idle
    }

    pub fn state(&self) -> ActivityState {
        if self.effective_idle >= self.threshold {
            ActivityState::Inactive
        } else {
            ActivityState::Active
        }
    }

    pub fn is_inactive(&self) -> bool {
        self.state() == ActivityState::Inactive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn becomes_inactive_once_threshold_reached() {
        let mut monitor = InactivityMonitor::new(Duration::from_secs(300));
        let t0 = Instant::now();

        monitor.poll(t0, Duration::from_secs(0));
        assert_eq!(monitor.state(), ActivityState::Active);

        monitor.poll(t0, Duration::from_secs(299));
        assert_eq!(monitor.state(), ActivityState::Active);

        monitor.poll(t0, Duration::from_secs(300));
        assert_eq!(monitor.state(), ActivityState::Inactive);
    }

    #[test]
    fn real_user_activity_resets_idle_immediately_outside_grace_window() {
        let mut monitor = InactivityMonitor::new(Duration::from_secs(300));
        let t0 = Instant::now();

        monitor.poll(t0, Duration::from_secs(400));
        assert!(monitor.is_inactive());

        // User moves the mouse: OS idle drops back to ~0, no synthetic action was recorded.
        let t1 = t0 + Duration::from_secs(401);
        monitor.poll(t1, Duration::from_secs(0));
        assert!(!monitor.is_inactive());
    }

    #[test]
    fn synthetic_action_within_grace_window_does_not_look_like_user_return() {
        let mut monitor =
            InactivityMonitor::with_grace_period(Duration::from_secs(300), Duration::from_secs(2));
        let t0 = Instant::now();

        monitor.poll(t0, Duration::from_secs(400));
        assert!(monitor.is_inactive());

        // Engine performs an automated click, which resets the *real* OS idle counter.
        monitor.notify_synthetic_action(t0);

        // Near-zero OS reading caused by us, but inside the grace window.
        let t1 = t0 + Duration::from_secs(1);
        monitor.poll(t1, Duration::from_secs(1));
        assert!(
            monitor.is_inactive(),
            "synthetic action must not reset inactivity state"
        );
    }

    #[test]
    fn genuine_activity_right_after_synthetic_action_is_still_caught_once_grace_expires() {
        let mut monitor =
            InactivityMonitor::with_grace_period(Duration::from_secs(300), Duration::from_secs(2));
        let t0 = Instant::now();

        monitor.poll(t0, Duration::from_secs(400));
        monitor.notify_synthetic_action(t0);

        // Still inside grace window.
        monitor.poll(t0 + Duration::from_secs(1), Duration::from_secs(1));
        assert!(monitor.is_inactive());

        // Grace window expired; a near-zero reading now means real activity.
        monitor.poll(t0 + Duration::from_secs(3), Duration::from_secs(0));
        assert!(!monitor.is_inactive());
    }
}
