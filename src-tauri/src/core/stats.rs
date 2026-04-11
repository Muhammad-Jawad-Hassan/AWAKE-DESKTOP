//! Local, opt-in activity statistics. Tracks durations and
//! counts only - never *what* the user typed or clicked.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    active_secs: u64,
    inactive_secs: u64,
    automated_event_count: u32,
    longest_inactive_secs: u64,
    current_inactive_streak_secs: u64,
}

impl SessionStats {
    /// Call once per tick with the tick length and whether the user was inactive during it.
    pub fn record_tick(&mut self, tick: Duration, user_inactive: bool) {
        let secs = tick.as_secs();
        if user_inactive {
            self.inactive_secs += secs;
            self.current_inactive_streak_secs += secs;
            self.longest_inactive_secs = self
                .longest_inactive_secs
                .max(self.current_inactive_streak_secs);
        } else {
            self.active_secs += secs;
            self.current_inactive_streak_secs = 0;
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

        assert_eq!(stats.active_secs, 2);
        assert_eq!(stats.inactive_secs, 1);
    }

    #[test]
    fn tracks_longest_inactive_streak_across_interruptions() {
        let mut stats = SessionStats::default();
        stats.record_tick(Duration::from_secs(5), true);
        stats.record_tick(Duration::from_secs(5), true);
        stats.record_tick(Duration::from_secs(1), false); // streak broken
        stats.record_tick(Duration::from_secs(3), true);

        assert_eq!(stats.longest_inactive_secs, 10);
        assert_eq!(stats.current_inactive_streak_secs, 3);
    }

    #[test]
    fn counts_automated_events() {
        let mut stats = SessionStats::default();
        stats.record_automated_event();
        stats.record_automated_event();
        assert_eq!(stats.automated_event_count, 2);
    }
}
