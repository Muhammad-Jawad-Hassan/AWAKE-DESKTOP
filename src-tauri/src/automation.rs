//! Performs planned activity through the input port. Blocking; run it off the async runtime.

use std::time::{Duration, Instant};

use rand::Rng;

use crate::core::activity::{
    jiggle_clear_of_corners, jiggle_offset, scroll_amount, split_path_steps,
};
use crate::core::ports::{InputSimulator, PlatformError};
use crate::core::{ActivityKind, ActivityProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Input was injected between these instants.
    Performed { started: Instant, ended: Instant },
    /// A safety check blocked it, e.g. the move would touch a screen corner.
    Skipped,
    /// Cancelled; any input already sent happened between these instants.
    Aborted { started: Instant, ended: Instant },
}

pub fn perform(
    input: &dyn InputSimulator,
    profile: &ActivityProfile,
    kind: ActivityKind,
    aborted: &dyn Fn() -> bool,
) -> Result<Outcome, PlatformError> {
    let started = Instant::now();
    if aborted() {
        return Ok(Outcome::Aborted {
            started,
            ended: started,
        });
    }
    let mut rng = rand::thread_rng();
    match kind {
        ActivityKind::MouseMovement => {
            let offset = jiggle_offset(&mut rng);
            let clear = jiggle_clear_of_corners(
                input.cursor_position()?,
                offset,
                input.display_size()?,
                profile.safety.avoid_screen_corners_px,
            );
            if !clear {
                return Ok(Outcome::Skipped);
            }
            if !jiggle(input, offset, aborted, &mut rng)? {
                return Ok(Outcome::Aborted {
                    started,
                    ended: Instant::now(),
                });
            }
        }
        ActivityKind::KeyboardInput => input.key_tap(profile.keyboard.next_key(&mut rng))?,
        ActivityKind::GestureHorizontal => input.scroll(scroll_amount(&mut rng), 0)?,
        ActivityKind::GestureVertical => input.scroll(0, scroll_amount(&mut rng))?,
    }
    Ok(Outcome::Performed {
        started,
        ended: Instant::now(),
    })
}

/// Moves out by `offset` and back in small steps, so the cursor ends where it began.
/// Returns `false` if aborted part way.
fn jiggle(
    input: &dyn InputSimulator,
    (dx, dy): (i32, i32),
    aborted: &dyn Fn() -> bool,
    rng: &mut impl Rng,
) -> Result<bool, PlatformError> {
    let out = split_path_steps(dx, dy, rng.gen_range(4..=7));
    let back = out.iter().rev().map(|(x, y)| (-x, -y));
    for (step_dx, step_dy) in out.iter().copied().chain(back) {
        if aborted() {
            return Ok(false);
        }
        input.move_mouse_relative(step_dx, step_dy)?;
        std::thread::sleep(Duration::from_millis(rng.gen_range(12..=28)));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use super::*;

    fn flag(f: &AtomicBool) -> impl Fn() -> bool + '_ {
        || f.load(Ordering::SeqCst)
    }

    #[derive(Default)]
    struct FakeInput {
        cursor: Mutex<(i32, i32)>,
        keys: Mutex<Vec<String>>,
        abort_after_moves: Option<(usize, &'static AtomicBool)>,
        moves: Mutex<usize>,
    }

    impl InputSimulator for FakeInput {
        fn move_mouse_relative(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
            let mut cursor = self.cursor.lock().unwrap();
            *cursor = (cursor.0 + dx, cursor.1 + dy);
            let mut moves = self.moves.lock().unwrap();
            *moves += 1;
            if let Some((after, flag)) = self.abort_after_moves {
                if *moves >= after {
                    flag.store(true, Ordering::SeqCst);
                }
            }
            Ok(())
        }
        fn key_tap(&self, key: &str) -> Result<(), PlatformError> {
            self.keys.lock().unwrap().push(key.to_string());
            Ok(())
        }
        fn scroll(&self, _dx: i32, _dy: i32) -> Result<(), PlatformError> {
            Ok(())
        }
        fn cursor_position(&self) -> Result<(i32, i32), PlatformError> {
            Ok(*self.cursor.lock().unwrap())
        }
        fn display_size(&self) -> Result<(i32, i32), PlatformError> {
            Ok((1920, 1080))
        }
    }

    fn profile() -> ActivityProfile {
        ActivityProfile::new_custom("p".to_string())
    }

    #[test]
    fn a_jiggle_returns_the_cursor_to_where_it_started() {
        let input = FakeInput {
            cursor: Mutex::new((500, 500)),
            ..FakeInput::default()
        };
        let abort = AtomicBool::new(false);
        for _ in 0..5 {
            let outcome = perform(
                &input,
                &profile(),
                ActivityKind::MouseMovement,
                &flag(&abort),
            )
            .unwrap();
            assert!(matches!(outcome, Outcome::Performed { .. }));
        }
        assert_eq!(*input.cursor.lock().unwrap(), (500, 500));
    }

    #[test]
    fn a_cursor_parked_in_a_corner_is_skipped_without_moving() {
        let input = FakeInput {
            cursor: Mutex::new((1, 1)),
            ..FakeInput::default()
        };
        let abort = AtomicBool::new(false);
        let outcome = perform(
            &input,
            &profile(),
            ActivityKind::MouseMovement,
            &flag(&abort),
        )
        .unwrap();
        assert_eq!(outcome, Outcome::Skipped);
        assert_eq!(*input.moves.lock().unwrap(), 0);
    }

    #[test]
    fn the_emergency_stop_cuts_a_jiggle_short() {
        static ABORT: AtomicBool = AtomicBool::new(false);
        let input = FakeInput {
            cursor: Mutex::new((500, 500)),
            abort_after_moves: Some((2, &ABORT)),
            ..FakeInput::default()
        };
        let outcome = perform(
            &input,
            &profile(),
            ActivityKind::MouseMovement,
            &flag(&ABORT),
        )
        .unwrap();
        assert!(matches!(outcome, Outcome::Aborted { .. }));
        assert_eq!(*input.moves.lock().unwrap(), 2);
    }

    #[test]
    fn nothing_runs_once_aborted() {
        let input = FakeInput::default();
        let abort = AtomicBool::new(true);
        let outcome = perform(
            &input,
            &profile(),
            ActivityKind::KeyboardInput,
            &flag(&abort),
        )
        .unwrap();
        assert!(matches!(outcome, Outcome::Aborted { .. }));
        assert!(input.keys.lock().unwrap().is_empty());
    }

    #[test]
    fn taps_the_configured_key() {
        let input = FakeInput::default();
        let mut profile = profile();
        profile.keyboard.key = "Shift".to_string();
        perform(&input, &profile, ActivityKind::KeyboardInput, &|| false).unwrap();
        assert_eq!(*input.keys.lock().unwrap(), vec!["Shift".to_string()]);
    }
}
