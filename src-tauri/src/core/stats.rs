//! Local, opt-in activity statistics. Tracks durations and
//! counts only - never *what* the user typed or clicked.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    #[serde(with = "super::serde_secs")]
    active_secs: Duration,
    #[serde(with = "super::serde_secs")]
    inactive_secs: Duration,
    automated_event_count: u32,
    #[serde(with = "super::serde_secs")]
    longest_inactive_secs: Duration,
    #[serde(with = "super::serde_secs")]
    current_inactive_streak_secs: Duration,
}

impl SessionStats {
    /// Credits `elapsed` to inactive or active time.
    pub fn record_tick(&mut self, elapsed: Duration, user_inactive: bool) {
        if user_inactive {
            self.inactive_secs += elapsed;
            self.current_inactive_streak_secs += elapsed;
            self.longest_inactive_secs = self
                .longest_inactive_secs
                .max(self.current_inactive_streak_secs);
        } else {
            self.active_secs += elapsed;
            self.current_inactive_streak_secs = Duration::ZERO;
        }
    }

    pub fn record_automated_event(&mut self) {
        self.automated_event_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_active_and_inactive_time_separately() {
        let mut stats = SessionStats::default();
        stats.record_tick(Duration::from_secs(1), false);
        stats.record_tick(Duration::from_secs(1), false);
        stats.record_tick(Duration::from_secs(1), true);

        assert_eq!(stats.active_secs, Duration::from_secs(2));
        assert_eq!(stats.inactive_secs, Duration::from_secs(1));
    }

    #[test]
    fn tracks_longest_inactive_streak_across_interruptions() {
        let mut stats = SessionStats::default();
        stats.record_tick(Duration::from_secs(5), true);
        stats.record_tick(Duration::from_secs(5), true);
        stats.record_tick(Duration::from_secs(1), false); // streak broken
        stats.record_tick(Duration::from_secs(3), true);

        assert_eq!(stats.longest_inactive_secs, Duration::from_secs(10));
        assert_eq!(stats.current_inactive_streak_secs, Duration::from_secs(3));
    }

    #[test]
    fn keeps_sub_second_time_instead_of_truncating_each_tick() {
        let mut stats = SessionStats::default();
        for _ in 0..10 {
            stats.record_tick(Duration::from_millis(1100), false);
        }
        assert_eq!(stats.active_secs, Duration::from_secs(11));
    }

    #[test]
    fn serializes_durations_as_whole_seconds() {
        let mut stats = SessionStats::default();
        stats.record_tick(Duration::from_millis(2500), true);
        let json = serde_json::to_value(stats).unwrap();
        assert_eq!(json["inactiveSecs"], 2);
        assert_eq!(
            serde_json::from_value::<SessionStats>(json)
                .unwrap()
                .inactive_secs,
            Duration::from_secs(2)
        );
    }

    #[test]
    fn counts_automated_events() {
        let mut stats = SessionStats::default();
        stats.record_automated_event();
        stats.record_automated_event();
        assert_eq!(stats.automated_event_count, 2);
    }
}
