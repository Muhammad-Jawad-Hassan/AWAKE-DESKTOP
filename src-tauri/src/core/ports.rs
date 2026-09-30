//! Ports: the interfaces the platform layer implements.
//! Core/runtime depend only on these traits, never on `platform::macos` etc.

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    // Only constructed on macOS.
    #[allow(dead_code)]
    #[error("operating system permission required: {0}")]
    PermissionDenied(String),
    #[error("not supported on this platform: {0}")]
    Unsupported(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// A held OS "prevent sleep" assertion; dropping it releases the assertion.
pub trait PowerLease: Send {
    /// Whether the assertion still holds (e.g. the backing process hasn't died).
    fn is_alive(&mut self) -> bool {
        true
    }
}

pub trait PowerManager: Send + Sync {
    /// `max_duration` lets a lease self-expire if the app dies without `Drop`.
    fn acquire(
        &self,
        keep_system_awake: bool,
        keep_display_awake: bool,
        max_duration: std::time::Duration,
    ) -> Result<Box<dyn PowerLease>, PlatformError>;
}

pub trait IdleProvider: Send + Sync {
    /// Seconds since the last real HID input event, as reported by the OS.
    fn idle_seconds(&self) -> Result<f64, PlatformError>;
}

pub trait InputSimulator: Send + Sync {
    fn move_mouse_relative(&self, dx: i32, dy: i32) -> Result<(), PlatformError>;
    fn key_tap(&self, key: &str) -> Result<(), PlatformError>;
    fn scroll(&self, dx: i32, dy: i32) -> Result<(), PlatformError>;
    fn cursor_position(&self) -> Result<(i32, i32), PlatformError>;
    fn display_size(&self) -> Result<(i32, i32), PlatformError>;
}

/// An opaque, platform-specific window handle passed down from the command layer.
pub enum WindowHandle {
    #[cfg(target_os = "macos")]
    MacOs(*mut std::ffi::c_void),
    #[cfg(target_os = "windows")]
    Windows(isize),
    #[cfg(target_os = "linux")]
    Linux,
}

pub trait VisibilityManager: Send + Sync {
    fn set_exclude_from_capture(
        &self,
        window: WindowHandle,
        exclude: bool,
    ) -> Result<(), PlatformError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub os: String,
    pub power_management: bool,
    pub idle_detection: bool,
    pub input_simulation: bool,
    /// Live: whether the OS lets us inject input right now (macOS Accessibility).
    pub input_permission_granted: bool,
    pub screen_capture_exclusion: bool,
    /// Explains any `false` capability above.
    pub notes: Vec<String>,
}
