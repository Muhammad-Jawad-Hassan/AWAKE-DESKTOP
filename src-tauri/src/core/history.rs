//! Bounded local history of completed sessions, gated by the
//! same opt-in `record_activity_statistics` setting as in-session stats.

use serde::{Deserialize, Serialize};

use super::stats::SessionStats;

/// Oldest entries are dropped once history exceeds this, so the config file never grows unbounded.
pub const MAX_HISTORY_ENTRIES: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub ended_at_unix_secs: u64,
    pub duration_secs: u64,
    pub activity_profile_id: Option<String>,
    pub stats: SessionStats,
}

pub fn push_bounded(history: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    history.push(entry);
    if history.len() > MAX_HISTORY_ENTRIES {
        let excess = history.len() - MAX_HISTORY_ENTRIES;
        history.drain(0..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            ended_at_unix_secs: 0,
            duration_secs: 60,
            activity_profile_id: None,
            stats: SessionStats::default(),
        }
    }

    #[test]
    fn keeps_all_entries_under_the_cap() {
        let mut history = Vec::new();
        for i in 0..10 {
            push_bounded(&mut history, entry(&i.to_string()));
        }
        assert_eq!(history.len(), 10);
    }

    #[test]
    fn drops_oldest_entries_beyond_the_cap() {
        let mut history = Vec::new();
        for i in 0..(MAX_HISTORY_ENTRIES + 5) {
            push_bounded(&mut history, entry(&i.to_string()));
        }
        assert_eq!(history.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(history.first().unwrap().id, "5");
        assert_eq!(
            history.last().unwrap().id,
            (MAX_HISTORY_ENTRIES + 4).to_string()
        );
    }
}
