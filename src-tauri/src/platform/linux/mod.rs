//! Linux platform implementation; experimental, capability-gated per feature.

mod idle;
mod power;
mod visibility;

use std::sync::Arc;

use crate::core::ports::PlatformCapabilities;

use super::input::EnigoInputSimulator;
use super::Platform;

pub fn build() -> Platform {
    let (input, input_ok) = EnigoInputSimulator::new();
    let power_ok = which::which("systemd-inhibit").is_ok();
    let idle_ok = which::which("xprintidle").is_ok();

    let mut notes = Vec::new();
    if !power_ok {
        notes.push("systemd-inhibit not found; sleep prevention is unavailable.".to_string());
    }
    if !idle_ok {
        notes.push(
            "xprintidle not found; inactivity detection needs X11 and xprintidle.".to_string(),
        );
    }
    if !input_ok {
        notes.push("Input simulation failed to initialize (may need X11 or XWayland).".to_string());
    }
    notes.push("Screen-capture exclusion is not supported on Linux.".to_string());

    Platform {
        power: Arc::new(power::LinuxPowerManager),
        idle: Arc::new(idle::LinuxIdleProvider),
        input: Arc::new(input),
        visibility: Arc::new(visibility::LinuxVisibilityManager),
        capabilities: PlatformCapabilities {
            os: "linux".to_string(),
            power_management: power_ok,
            idle_detection: idle_ok,
            input_simulation: input_ok,
            screen_capture_exclusion: false,
            notes,
        },
    }
}
