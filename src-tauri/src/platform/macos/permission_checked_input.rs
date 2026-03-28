//! Wraps an `InputSimulator`, failing fast with `PermissionDenied` when
//! Accessibility access hasn't been granted, instead of a vague OS error.

use crate::core::ports::{InputSimulator, PlatformError};
use crate::core::{ClickKind, MouseButton};

use super::permissions::is_accessibility_trusted;

pub struct PermissionCheckedInputSimulator<T> {
    inner: T,
}

impl<T> PermissionCheckedInputSimulator<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    fn check(&self) -> Result<(), PlatformError> {
        if is_accessibility_trusted() {
            Ok(())
        } else {
            Err(PlatformError::PermissionDenied(
                "Accessibility access not granted".into(),
            ))
        }
    }
}

impl<T: InputSimulator> InputSimulator for PermissionCheckedInputSimulator<T> {
    fn move_mouse_relative(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
        self.check()?;
        self.inner.move_mouse_relative(dx, dy)
    }

    fn click_mouse(&self, button: MouseButton, click: ClickKind) -> Result<(), PlatformError> {
        self.check()?;
        self.inner.click_mouse(button, click)
    }

    fn key_tap(&self, key: &str, modifiers: &[String]) -> Result<(), PlatformError> {
        self.check()?;
        self.inner.key_tap(key, modifiers)
    }

    fn scroll(&self, dx: i32, dy: i32) -> Result<(), PlatformError> {
        self.check()?;
        self.inner.scroll(dx, dy)
    }

    fn cursor_position(&self) -> Result<(i32, i32), PlatformError> {
        self.inner.cursor_position()
    }

    fn display_size(&self) -> Result<(i32, i32), PlatformError> {
        self.inner.display_size()
    }
}
