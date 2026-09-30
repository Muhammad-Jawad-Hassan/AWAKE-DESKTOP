//! Shared `enigo`-backed input simulator, used by all three platforms.

use std::sync::Mutex;

use enigo::{Axis, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

use crate::core::ports::{InputSimulator, PlatformError};

/// Created lazily and retried, since macOS refuses it until Accessibility is granted.
pub struct EnigoInputSimulator {
    enigo: Mutex<Option<Enigo>>,
}

impl EnigoInputSimulator {
    pub fn new() -> Self {
        Self {
            enigo: Mutex::new(Enigo::new(&settings()).ok()),
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn is_available(&self) -> bool {
        self.with_enigo(|_| Ok(())).is_ok()
    }

    fn with_enigo<T>(
        &self,
        f: impl FnOnce(&mut Enigo) -> enigo::InputResult<T>,
    ) -> Result<T, PlatformError> {
        // Recover from a poisoned lock.
        let mut guard = self
            .enigo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.is_none() {
            let enigo = Enigo::new(&settings()).map_err(|e| {
                PlatformError::Unsupported(format!("input simulation unavailable: {e}"))
            })?;
            *guard = Some(enigo);
        }
        let enigo = guard.as_mut().expect("initialized above");
        f(enigo).map_err(|e| PlatformError::OperationFailed(e.to_string()))
    }
}

fn settings() -> Settings {
    Settings {
        // We show our own prompt.
        open_prompt_to_get_permissions: false,
        // Relative moves, not primary-monitor absolute.
        windows_subject_to_mouse_speed_and_acceleration_level: true,
        ..Settings::default()
    }
}

impl InputSimulator for EnigoInputSimulator {
    fn move_mouse_relative(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
        self.with_enigo(|e| e.move_mouse(dx, dy, Coordinate::Rel))
    }

    fn key_tap(&self, key: &str) -> Result<(), PlatformError> {
        let key = safe_key(key)
            .ok_or_else(|| PlatformError::OperationFailed(format!("refusing unsafe key: {key}")))?;
        self.with_enigo(|e| e.key(key, Direction::Click))
    }

    fn scroll(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
        self.with_enigo(|e| {
            if dx != 0 {
                e.scroll(dx, Axis::Horizontal)?;
            }
            if dy != 0 {
                e.scroll(dy, Axis::Vertical)?;
            }
            Ok(())
        })
    }

    fn cursor_position(&self) -> Result<(i32, i32), PlatformError> {
        self.with_enigo(|e| e.location())
    }

    fn display_size(&self) -> Result<(i32, i32), PlatformError> {
        self.with_enigo(|e| e.main_display())
    }
}

/// Maps a safe automation key to an `enigo::Key`. This match is the allowlist.
fn safe_key(name: &str) -> Option<Key> {
    match name.trim().to_ascii_lowercase().as_str() {
        "shift" => Some(Key::Shift),
        "control" => Some(Key::Control),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::profiles::SAFE_AUTOMATION_KEYS;

    #[test]
    fn every_safe_key_maps_case_insensitively() {
        for key in SAFE_AUTOMATION_KEYS {
            assert!(safe_key(key).is_some(), "{key}");
            assert!(safe_key(&key.to_uppercase()).is_some(), "{key}");
        }
    }

    #[test]
    fn refuses_anything_outside_the_safe_list() {
        for key in [
            "a", "Enter", "Tab", "Alt", "Meta", "F5", "F13", "Random", "",
        ] {
            assert_eq!(safe_key(key), None, "{key:?}");
        }
    }
}
