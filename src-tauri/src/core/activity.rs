//! Activity planning: what to do and when. The platform layer performs the input.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::profiles::{ActivityProfile, MAX_PER_MINUTE};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    MouseMovement,
    KeyboardInput,
    GestureHorizontal,
    GestureVertical,
}

/// The activity kinds a profile has switched on.
pub fn enabled_activities(profile: &ActivityProfile) -> Vec<ActivityKind> {
    [
        (profile.mouse.enabled, ActivityKind::MouseMovement),
        (profile.keyboard.enabled, ActivityKind::KeyboardInput),
        (profile.gestures.horizontal, ActivityKind::GestureHorizontal),
        (profile.gestures.vertical, ActivityKind::GestureVertical),
    ]
    .into_iter()
    .filter_map(|(on, kind)| on.then_some(kind))
    .collect()
}

/// How often `kind` fires per minute; the scroll rate is split across enabled axes.
pub fn per_minute(profile: &ActivityProfile, kind: ActivityKind) -> f64 {
    let clamp = |rate: u32| f64::from(rate.clamp(1, MAX_PER_MINUTE));
    match kind {
        ActivityKind::MouseMovement => clamp(profile.mouse.per_minute),
        ActivityKind::KeyboardInput => clamp(profile.keyboard.per_minute),
        ActivityKind::GestureHorizontal | ActivityKind::GestureVertical => {
            let axes =
                u32::from(profile.gestures.horizontal) + u32::from(profile.gestures.vertical);
            clamp(profile.gestures.per_minute) / f64::from(axes.max(1))
        }
    }
}

/// A delay averaging 60/`per_minute` seconds, jittered +/-50%.
pub fn jittered_interval(per_minute: f64, rng: &mut impl Rng) -> Duration {
    let mean_ms = 60_000.0 / per_minute;
    Duration::from_millis(rng.gen_range(mean_ms * 0.5..=mean_ms * 1.5) as u64)
}

/// One independent, jittered timer per enabled activity kind.
#[derive(Debug, Default)]
pub struct ActivityScheduler {
    next_due: HashMap<ActivityKind, Instant>,
}

impl ActivityScheduler {
    pub fn reset(&mut self) {
        self.next_due.clear();
    }

    /// Kinds due at `now`, most overdue first. Newly enabled kinds are scheduled, not fired.
    pub fn due(
        &mut self,
        profile: &ActivityProfile,
        now: Instant,
        rng: &mut impl Rng,
    ) -> Vec<ActivityKind> {
        let enabled = enabled_activities(profile);
        self.next_due.retain(|kind, _| enabled.contains(kind));
        for kind in &enabled {
            self.next_due
                .entry(*kind)
                .or_insert_with(|| now + jittered_interval(per_minute(profile, *kind), rng));
        }
        let mut due: Vec<_> = self
            .next_due
            .iter()
            .filter(|(_, at)| **at <= now)
            .map(|(kind, at)| (*at, *kind))
            .collect();
        due.sort();
        due.into_iter().map(|(_, kind)| kind).collect()
    }

    /// Schedules `kind`'s next run after it ran at `now`, keeping the mean rate exact.
    pub fn complete(
        &mut self,
        kind: ActivityKind,
        profile: &ActivityProfile,
        now: Instant,
        rng: &mut impl Rng,
    ) {
        let interval = jittered_interval(per_minute(profile, kind), rng);
        let anchor = match self.next_due.get(&kind) {
            Some(at) if now.saturating_duration_since(*at) < interval => *at,
            _ => now,
        };
        self.next_due.insert(kind, anchor + interval);
    }
}

/// Sliding 60-second cap on automated actions.
#[derive(Debug, Default)]
pub struct RateLimiter {
    recent: VecDeque<Instant>,
}

impl RateLimiter {
    const WINDOW: Duration = Duration::from_secs(60);

    pub fn reset(&mut self) {
        self.recent.clear();
    }

    /// Takes a slot if fewer than `cap` actions ran in the last minute.
    pub fn try_acquire(&mut self, now: Instant, cap: u32) -> bool {
        while self
            .recent
            .front()
            .is_some_and(|at| now.saturating_duration_since(*at) >= Self::WINDOW)
        {
            self.recent.pop_front();
        }
        if self.recent.len() >= cap as usize {
            return false;
        }
        self.recent.push_back(now);
        true
    }
}

/// A nonzero scroll amount of up to five notches either way.
pub fn scroll_amount(rng: &mut impl Rng) -> i32 {
    let notches = rng.gen_range(1..=5);
    if rng.gen_bool(0.5) {
        notches
    } else {
        -notches
    }
}

/// A random offset for an out-and-back mouse jiggle, never (0, 0).
pub fn jiggle_offset(rng: &mut impl Rng) -> (i32, i32) {
    loop {
        let offset = (rng.gen_range(-40..=40), rng.gen_range(-40..=40));
        if offset != (0, 0) {
            return offset;
        }
    }
}

/// Whether a jiggle from `start` by `offset` stays clear of every display corner.
pub fn jiggle_clear_of_corners(
    start: (i32, i32),
    offset: (i32, i32),
    display: (i32, i32),
    margin: u32,
) -> bool {
    let end = (
        start.0.saturating_add(offset.0),
        start.1.saturating_add(offset.1),
    );
    [start, end]
        .into_iter()
        .all(|point| !near_corner(point, display, margin))
}

fn near_corner((x, y): (i32, i32), (width, height): (i32, i32), margin: u32) -> bool {
    let margin = i32::try_from(margin).unwrap_or(i32::MAX);
    let near_x = x < margin || x >= width.saturating_sub(margin);
    let near_y = y < margin || y >= height.saturating_sub(margin);
    near_x && near_y
}

/// Splits a relative move into steps summing to the original distance.
pub fn split_path_steps(total_dx: i32, total_dy: i32, steps: u32) -> Vec<(i32, i32)> {
    let steps = steps.max(1);
    let mut remaining_dx = total_dx;
    let mut remaining_dy = total_dy;
    let mut out = Vec::with_capacity(steps as usize);
    for step in 1..=steps {
        let steps_left = (steps - step + 1) as i32;
        let step_dx = remaining_dx / steps_left;
        let step_dy = remaining_dy / steps_left;
        out.push((step_dx, step_dy));
        remaining_dx -= step_dx;
        remaining_dy -= step_dy;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::profiles::ActivityProfile;
    use rand::{rngs::StdRng, SeedableRng};

    fn profile_with(mouse: u32, keys: u32) -> ActivityProfile {
        let mut profile = ActivityProfile::new_custom("p".to_string());
        profile.mouse.per_minute = mouse;
        profile.keyboard.enabled = keys > 0;
        profile.keyboard.per_minute = keys.max(1);
        profile.safety.max_actions_per_minute = 30;
        profile
    }

    /// Ticks once a second like the runtime and counts what actually runs.
    fn simulate(profile: &ActivityProfile, minutes: u64) -> HashMap<ActivityKind, u32> {
        let mut rng = StdRng::seed_from_u64(11);
        let mut scheduler = ActivityScheduler::default();
        let mut limiter = RateLimiter::default();
        let mut counts = HashMap::new();
        let start = Instant::now();
        for sec in 0..minutes * 60 {
            let now = start + Duration::from_secs(sec);
            for kind in scheduler.due(profile, now, &mut rng) {
                if !limiter.try_acquire(now, profile.safety.max_actions_per_minute) {
                    break;
                }
                scheduler.complete(kind, profile, now, &mut rng);
                *counts.entry(kind).or_insert(0) += 1;
            }
        }
        counts
    }

    #[test]
    fn jittered_interval_stays_within_half_to_one_and_a_half_of_the_mean() {
        let mut rng = StdRng::seed_from_u64(5);
        for _ in 0..1000 {
            let d = jittered_interval(4.0, &mut rng);
            assert!(d >= Duration::from_millis(7500) && d <= Duration::from_millis(22500));
        }
    }

    #[test]
    fn each_kind_fires_at_its_own_configured_rate() {
        let counts = simulate(&profile_with(6, 2), 60);
        let mouse = counts[&ActivityKind::MouseMovement];
        let keys = counts[&ActivityKind::KeyboardInput];
        assert!(
            (345..=375).contains(&mouse),
            "mouse fired {mouse} times in an hour"
        );
        assert!(
            (112..=128).contains(&keys),
            "keys fired {keys} times in an hour"
        );
    }

    #[test]
    fn one_activity_can_use_the_whole_cap() {
        let mut profile = profile_with(60, 0);
        profile.safety.max_actions_per_minute = 60;
        let mouse = simulate(&profile, 10)[&ActivityKind::MouseMovement];
        assert!(
            (540..=600).contains(&mouse),
            "mouse fired {mouse} times in 10 minutes"
        );
    }

    #[test]
    fn a_tight_cap_defers_actions_instead_of_dropping_them() {
        let mut profile = profile_with(4, 2);
        profile.safety.max_actions_per_minute = 6;
        let counts = simulate(&profile, 60);
        let total: u32 = counts.values().sum();
        assert!((340..=360).contains(&total), "ran {total} of ~360 actions");
        assert!(counts[&ActivityKind::KeyboardInput] >= 110);
    }

    #[test]
    fn first_sighting_schedules_instead_of_firing() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut scheduler = ActivityScheduler::default();
        assert!(scheduler
            .due(&profile_with(10, 10), Instant::now(), &mut rng)
            .is_empty());
    }

    #[test]
    fn a_kind_stays_due_until_completed() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut scheduler = ActivityScheduler::default();
        let profile = profile_with(10, 0);
        let start = Instant::now();
        scheduler.due(&profile, start, &mut rng);
        let later = start + Duration::from_secs(60);
        assert_eq!(scheduler.due(&profile, later, &mut rng).len(), 1);
        assert_eq!(scheduler.due(&profile, later, &mut rng).len(), 1);
        scheduler.complete(ActivityKind::MouseMovement, &profile, later, &mut rng);
        assert!(scheduler.due(&profile, later, &mut rng).is_empty());
    }

    #[test]
    fn a_long_stall_does_not_cause_a_catch_up_burst() {
        let mut rng = StdRng::seed_from_u64(2);
        let mut scheduler = ActivityScheduler::default();
        let profile = profile_with(10, 0);
        let start = Instant::now();
        scheduler.due(&profile, start, &mut rng);
        let after_sleep = start + Duration::from_secs(3600);
        scheduler.complete(ActivityKind::MouseMovement, &profile, after_sleep, &mut rng);
        assert!(scheduler
            .due(&profile, after_sleep + Duration::from_secs(1), &mut rng)
            .is_empty());
    }

    #[test]
    fn disabling_a_kind_drops_its_pending_timer() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut scheduler = ActivityScheduler::default();
        let start = Instant::now();
        scheduler.due(&profile_with(1, 1), start, &mut rng);
        let later = start + Duration::from_secs(600);
        assert_eq!(
            scheduler.due(&profile_with(1, 0), later, &mut rng),
            vec![ActivityKind::MouseMovement]
        );
    }

    #[test]
    fn scroll_rate_is_shared_between_both_axes() {
        let mut profile = profile_with(1, 0);
        profile.gestures.horizontal = true;
        profile.gestures.vertical = true;
        profile.gestures.per_minute = 4;
        assert_eq!(per_minute(&profile, ActivityKind::GestureHorizontal), 2.0);
        profile.gestures.vertical = false;
        assert_eq!(per_minute(&profile, ActivityKind::GestureHorizontal), 4.0);
    }

    #[test]
    fn a_zero_rate_is_treated_as_the_minimum_instead_of_panicking() {
        let mut profile = profile_with(1, 0);
        profile.mouse.per_minute = 0;
        let mut rng = StdRng::seed_from_u64(1);
        let interval =
            jittered_interval(per_minute(&profile, ActivityKind::MouseMovement), &mut rng);
        assert!(interval <= Duration::from_secs(90));
    }

    #[test]
    fn rate_limiter_caps_actions_per_rolling_minute() {
        let mut limiter = RateLimiter::default();
        let start = Instant::now();
        assert!(limiter.try_acquire(start, 2));
        assert!(limiter.try_acquire(start, 2));
        assert!(!limiter.try_acquire(start + Duration::from_secs(59), 2));
        assert!(limiter.try_acquire(start + Duration::from_secs(60), 2));
    }

    #[test]
    fn scroll_amount_is_never_zero() {
        let mut rng = StdRng::seed_from_u64(4);
        let amounts: std::collections::HashSet<_> =
            (0..500).map(|_| scroll_amount(&mut rng)).collect();
        assert!(!amounts.contains(&0));
        assert_eq!(amounts.len(), 10);
    }

    #[test]
    fn jiggle_offset_is_never_zero() {
        let mut rng = StdRng::seed_from_u64(9);
        assert!((0..500).all(|_| jiggle_offset(&mut rng) != (0, 0)));
    }

    #[test]
    fn a_jiggle_ending_in_a_corner_is_rejected() {
        let display = (1920, 1080);
        assert!(!jiggle_clear_of_corners((30, 30), (-40, -40), display, 24));
        assert!(!jiggle_clear_of_corners((1900, 1070), (0, 0), display, 24));
        assert!(jiggle_clear_of_corners((30, 30), (20, 20), display, 24));
        assert!(jiggle_clear_of_corners((10, 500), (0, 0), display, 24));
    }

    #[test]
    fn a_huge_margin_blocks_rather_than_wrapping() {
        assert!(!jiggle_clear_of_corners(
            (500, 500),
            (1, 1),
            (1920, 1080),
            u32::MAX
        ));
    }

    #[test]
    fn path_steps_sum_to_the_original_distance() {
        for (dx, dy) in [(40, -40), (-17, 3), (0, 0), (1, -1), (40, 40)] {
            let steps = split_path_steps(dx, dy, 6);
            let (sum_dx, sum_dy) = steps
                .iter()
                .fold((0, 0), |(ax, ay), (x, y)| (ax + x, ay + y));
            assert_eq!((sum_dx, sum_dy), (dx, dy));
        }
    }

    #[test]
    fn path_steps_returns_the_requested_count() {
        assert_eq!(split_path_steps(40, -40, 5).len(), 5);
        assert_eq!(split_path_steps(10, -10, 0), vec![(10, -10)]);
    }

    #[test]
    fn enabled_activities_reflects_profile_flags() {
        let mut profile = profile_with(1, 0);
        profile.gestures.horizontal = true;
        assert_eq!(
            enabled_activities(&profile),
            vec![ActivityKind::MouseMovement, ActivityKind::GestureHorizontal]
        );
    }
}
