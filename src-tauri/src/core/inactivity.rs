//! Inactivity tracking against a configurable threshold.

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityState {
    Active,
    Inactive,
}

#[derive(Debug, Clone, Copy)]
pub struct InactivityMonitor {
    threshold: Duration,
    idle: Duration,
}

impl InactivityMonitor {
    pub fn new(threshold: Duration) -> Self {
        Self {
            threshold,
            idle: Duration::ZERO,
        }
    }

    /// Feeds a fresh OS idle reading and returns it.
    pub fn poll(&mut self, os_idle: Duration) -> Duration {
        self.idle = os_idle;
        self.idle
    }

    pub fn effective_idle(&self) -> Duration {
        self.idle
    }

    pub fn state(&self) -> ActivityState {
        if self.idle >= self.threshold {
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

        monitor.poll(Duration::from_secs(0));
        assert_eq!(monitor.state(), ActivityState::Active);

        monitor.poll(Duration::from_secs(299));
        assert_eq!(monitor.state(), ActivityState::Active);

        monitor.poll(Duration::from_secs(300));
        assert_eq!(monitor.state(), ActivityState::Inactive);
    }

    #[test]
    fn user_activity_resets_idle_to_zero() {
        let mut monitor = InactivityMonitor::new(Duration::from_secs(300));

        monitor.poll(Duration::from_secs(400));
        assert!(monitor.is_inactive());

        monitor.poll(Duration::from_secs(0));
        assert!(!monitor.is_inactive());
    }
}
