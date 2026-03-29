//! Windows platform implementation.

mod idle;
mod power;
mod visibility;

use std::sync::Arc;

use crate::core::ports::PlatformCapabilities;

use super::input::EnigoInputSimulator;
use super::Platform;

pub fn build() -> Platform {
    let (input, input_ok) = EnigoInputSimulator::new();
    let mut notes = Vec::new();
    if !input_ok {
        notes.push("Failed to initialize input simulation.".to_string());
    }

    Platform {
        power: Arc::new(power::WindowsPowerManager),
        idle: Arc::new(idle::WindowsIdleProvider),
        input: Arc::new(input),
        visibility: Arc::new(visibility::WindowsVisibilityManager),
        capabilities: PlatformCapabilities {
            os: "windows".to_string(),
            power_management: true,
            idle_detection: true,
            input_simulation: input_ok,
            screen_capture_exclusion: true,
            notes,
        },
    }
}
