//! Shared `enigo`-backed input simulator, used by all three platforms.

use std::sync::Mutex;

use enigo::{
    Axis, Button as EnigoButton, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings,
};

use crate::core::ports::{InputSimulator, PlatformError};
use crate::core::{ClickKind, MouseButton};

pub struct EnigoInputSimulator {
    enigo: Mutex<Option<Enigo>>,
}

impl EnigoInputSimulator {
    /// Returns the simulator plus whether it initialized successfully.
    pub fn new() -> (Self, bool) {
        match Enigo::new(&Settings::default()) {
            Ok(enigo) => (
                Self {
                    enigo: Mutex::new(Some(enigo)),
                },
                true,
            ),
            Err(_) => (
                Self {
                    enigo: Mutex::new(None),
                },
                false,
            ),
        }
    }

    fn with_enigo<T>(
        &self,
        f: impl FnOnce(&mut Enigo) -> enigo::InputResult<T>,
    ) -> Result<T, PlatformError> {
        // Recover from poisoning: one bad call shouldn't disable input for the app's lifetime.
        let mut guard = self
            .enigo
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let enigo = guard
            .as_mut()
            .ok_or_else(|| PlatformError::Unsupported("input simulation unavailable".into()))?;
        f(enigo).map_err(|e| PlatformError::OperationFailed(e.to_string()))
    }
}

impl InputSimulator for EnigoInputSimulator {
    fn move_mouse_relative(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
        self.with_enigo(|e| e.move_mouse(dx, dy, Coordinate::Rel))
    }

    fn click_mouse(&self, button: MouseButton, click: ClickKind) -> Result<(), PlatformError> {
        let button = match button {
            MouseButton::Left => EnigoButton::Left,
            MouseButton::Right => EnigoButton::Right,
            MouseButton::Middle => EnigoButton::Middle,
        };
        self.with_enigo(|e| {
            press_and_release(e, button)?;
            if click == ClickKind::Double {
                press_and_release(e, button)?;
            }
            Ok(())
        })
    }

    fn key_tap(&self, key: &str, modifiers: &[String]) -> Result<(), PlatformError> {
        let key = parse_key(key)
            .ok_or_else(|| PlatformError::OperationFailed(format!("unknown key: {key}")))?;
        let modifiers: Vec<Key> = modifiers.iter().filter_map(|m| parse_key(m)).collect();

        self.with_enigo(|e| {
            let press_result = modifiers
                .iter()
                .try_for_each(|modifier| e.key(*modifier, Direction::Press));
            let click_result = press_result.and_then(|()| e.key(key, Direction::Click));
            // Released even if the press or click failed, so an input error never leaves a modifier stuck down.
            for modifier in modifiers.iter().rev() {
                let _ = e.key(*modifier, Direction::Release);
            }
            click_result
        })
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

/// Presses then releases a button, always attempting the release even if the
/// press failed, so a transient error never leaves it stuck held down.
fn press_and_release(e: &mut Enigo, button: EnigoButton) -> enigo::InputResult<()> {
    let press = e.button(button, Direction::Press);
    let release = e.button(button, Direction::Release);
    press.and(release)
}

/// Maps a user-facing key name to an `enigo::Key`, case-insensitively.
fn parse_key(name: &str) -> Option<Key> {
    let normalized = name.trim();
    if let Some(key) = named_key(normalized) {
        return Some(key);
    }
    let mut chars = normalized.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Some(Key::Unicode(c)),
        _ => None,
    }
}

fn named_key(name: &str) -> Option<Key> {
    let key = match name.to_ascii_lowercase().as_str() {
        "shift" => Key::Shift,
        "control" | "ctrl" => Key::Control,
        "alt" | "option" => Key::Alt,
        "meta" | "cmd" | "command" | "super" | "win" => Key::Meta,
        "space" => Key::Space,
        "return" | "enter" => Key::Return,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "f3" => Key::F3,
        "f4" => Key::F4,
        "f5" => Key::F5,
        "f6" => Key::F6,
        "f7" => Key::F7,
        "f8" => Key::F8,
        "f9" => Key::F9,
        "f10" => Key::F10,
        "f11" => Key::F11,
        "f12" => Key::F12,
        "f13" => Key::F13,
        "f14" => Key::F14,
        "f15" => Key::F15,
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_keys_case_insensitively() {
        assert_eq!(parse_key("Shift"), Some(Key::Shift));
        assert_eq!(parse_key("SHIFT"), Some(Key::Shift));
        assert_eq!(parse_key("f15"), Some(Key::F15));
    }

    #[test]
    fn parses_single_character_as_unicode() {
        assert_eq!(parse_key("a"), Some(Key::Unicode('a')));
    }

    #[test]
    fn rejects_unknown_multi_character_input() {
        assert_eq!(parse_key("not-a-key"), None);
    }
}
