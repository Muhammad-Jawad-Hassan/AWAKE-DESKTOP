//! macOS platform implementation.

mod idle;
mod permission_checked_input;
mod permissions;
mod power;
mod visibility;

use std::sync::Arc;

use crate::core::ports::PlatformCapabilities;

use super::input::EnigoInputSimulator;
use super::Platform;
use permission_checked_input::PermissionCheckedInputSimulator;

pub use permissions::{is_accessibility_trusted, open_accessibility_settings};

pub fn build() -> Platform {
    let input = EnigoInputSimulator::new();

    Platform {
        power: Arc::new(power::MacOsPowerManager),
        idle: Arc::new(idle::MacOsIdleProvider),
        input: Arc::new(PermissionCheckedInputSimulator::new(input)),
        visibility: Arc::new(visibility::MacOsVisibilityManager),
        capabilities: PlatformCapabilities {
            os: "macos".to_string(),
            power_management: true,
            idle_detection: true,
            // Gated per call by Accessibility.
            input_simulation: true,
            input_permission_granted: is_accessibility_trusted(),
            screen_capture_exclusion: true,
            notes: Vec::new(),
        },
        input_permission_granted: is_accessibility_trusted,
    }
}
