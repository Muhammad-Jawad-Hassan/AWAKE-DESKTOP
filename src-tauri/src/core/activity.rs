//! Randomized activity scheduling.
//! Decides what/when only; the platform layer performs the actual input.

use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};

use super::profiles::ActivityProfile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    MouseMovement,
    MouseClick,
    KeyboardInput,
    GestureHorizontal,
    GestureVertical,
}

/// The set of activity kinds a profile currently has switched on.
pub fn enabled_activities(profile: &ActivityProfile) -> Vec<ActivityKind> {
    let mut kinds = Vec::new();
    if profile.mouse.enabled {
        if profile.mouse.movement {
            kinds.push(ActivityKind::MouseMovement);
        }
        kinds.push(ActivityKind::MouseClick);
    }
    if profile.keyboard.enabled {
        kinds.push(ActivityKind::KeyboardInput);
    }
    if profile.gestures.horizontal {
        kinds.push(ActivityKind::GestureHorizontal);
    }
    if profile.gestures.vertical {
        kinds.push(ActivityKind::GestureVertical);
    }
    kinds
}

/// Picks the next activity at random from the enabled set.
pub fn choose_activity(enabled: &[ActivityKind], rng: &mut impl Rng) -> Option<ActivityKind> {
    if enabled.is_empty() {
        return None;
    }
    let index = rng.gen_range(0..enabled.len());
    enabled.get(index).copied()
}

/// Generates the next randomized delay in `[min, max]`.
pub fn next_delay(min: Duration, max: Duration, rng: &mut impl Rng) -> Duration {
    if min >= max {
        return min;
    }
    let min_ms = min.as_millis() as u64;
    let max_ms = max.as_millis() as u64;
    Duration::from_millis(rng.gen_range(min_ms..=max_ms))
}

/// Splits a relative mouse move into smaller steps summing to the original
/// distance, so the cursor traces a path instead of teleporting.
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityOutcome {
    Performed(ActivityKind),
    /// The user returned before the scheduled activity fired.
    SkippedUserReturned,
    /// The profile has no activity types enabled.
    SkippedNothingEnabled,
}

/// Decides whether/what to perform when a scheduled delay elapses.
pub fn decide_outcome(
    enabled: &[ActivityKind],
    user_is_currently_inactive: bool,
    rng: &mut impl Rng,
) -> ActivityOutcome {
    if !user_is_currently_inactive {
        return ActivityOutcome::SkippedUserReturned;
    }
    match choose_activity(enabled, rng) {
        Some(kind) => ActivityOutcome::Performed(kind),
        None => ActivityOutcome::SkippedNothingEnabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn delay_is_always_within_bounds() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..1000 {
            let d = next_delay(Duration::from_secs(30), Duration::from_secs(180), &mut rng);
            assert!(d >= Duration::from_secs(30));
            assert!(d <= Duration::from_secs(180));
        }
    }

    #[test]
    fn delay_varies_across_calls() {
        let mut rng = StdRng::seed_from_u64(7);
        let samples: Vec<_> = (0..20)
            .map(|_| next_delay(Duration::from_secs(1), Duration::from_secs(1000), &mut rng))
            .collect();
        let unique: std::collections::HashSet<_> = samples.iter().collect();
        assert!(
            unique.len() > 1,
            "successive delays should not all be identical"
        );
    }

    #[test]
    fn degenerate_range_returns_fixed_delay() {
        let mut rng = StdRng::seed_from_u64(1);
        let d = next_delay(Duration::from_secs(5), Duration::from_secs(5), &mut rng);
        assert_eq!(d, Duration::from_secs(5));
    }

    #[test]
    fn choose_activity_only_returns_enabled_kinds() {
        let enabled = vec![ActivityKind::MouseMovement, ActivityKind::KeyboardInput];
        let mut rng = StdRng::seed_from_u64(99);
        for _ in 0..100 {
            let chosen = choose_activity(&enabled, &mut rng).unwrap();
            assert!(enabled.contains(&chosen));
        }
    }

    #[test]
    fn choose_activity_on_empty_set_returns_none() {
        let mut rng = StdRng::seed_from_u64(1);
        assert_eq!(choose_activity(&[], &mut rng), None);
    }

    #[test]
    fn user_return_cancels_pending_activity() {
        let enabled = vec![ActivityKind::MouseMovement];
        let mut rng = StdRng::seed_from_u64(3);
        let outcome = decide_outcome(&enabled, false, &mut rng);
        assert_eq!(outcome, ActivityOutcome::SkippedUserReturned);
    }

    #[test]
    fn inactive_user_with_enabled_activity_performs_one() {
        let enabled = vec![ActivityKind::KeyboardInput];
        let mut rng = StdRng::seed_from_u64(3);
        let outcome = decide_outcome(&enabled, true, &mut rng);
        assert_eq!(
            outcome,
            ActivityOutcome::Performed(ActivityKind::KeyboardInput)
        );
    }

    #[test]
    fn inactive_user_with_nothing_enabled_is_reported_distinctly() {
        let mut rng = StdRng::seed_from_u64(3);
        let outcome = decide_outcome(&[], true, &mut rng);
        assert_eq!(outcome, ActivityOutcome::SkippedNothingEnabled);
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
    }

    #[test]
    fn path_steps_treats_zero_as_one_step() {
        assert_eq!(split_path_steps(10, -10, 0), vec![(10, -10)]);
    }

    #[test]
    fn enabled_activities_reflects_profile_flags() {
        let mut profile = super::super::profiles::built_in_profiles().remove(0);
        profile.mouse.enabled = true;
        profile.mouse.movement = true;
        profile.keyboard.enabled = false;
        profile.gestures.horizontal = true;
        profile.gestures.vertical = false;

        let kinds = enabled_activities(&profile);
        assert!(kinds.contains(&ActivityKind::MouseMovement));
        assert!(kinds.contains(&ActivityKind::MouseClick));
        assert!(kinds.contains(&ActivityKind::GestureHorizontal));
        assert!(!kinds.contains(&ActivityKind::KeyboardInput));
        assert!(!kinds.contains(&ActivityKind::GestureVertical));
    }
}
