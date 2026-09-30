//! Linux platform implementation; experimental, capability-gated per feature.

mod idle;
mod power;
mod visibility;

use std::sync::Arc;

use crate::core::ports::PlatformCapabilities;

use super::input::EnigoInputSimulator;
use super::Platform;

pub fn build() -> Platform {
    let input = EnigoInputSimulator::new();
    // XTest and xprintidle miss Wayland clients.
    let wayland =
        std::env::var("XDG_SESSION_TYPE").is_ok_and(|t| t.eq_ignore_ascii_case("wayland"));
    let input_ok = !wayland && input.is_available();
    let power_ok = which::which("systemd-inhibit").is_ok();
    let idle_ok = !wayland && which::which("xprintidle").is_ok();

    let mut notes = Vec::new();
    if wayland {
        notes.push(
            "Wayland session: inactivity detection and activity automation need an X11 session."
                .to_string(),
        );
    }
    if !power_ok {
        notes.push("systemd-inhibit not found; sleep prevention is unavailable.".to_string());
    }
    if !idle_ok && !wayland {
        notes.push(
            "xprintidle not found; inactivity detection needs X11 and xprintidle.".to_string(),
        );
    }
    if !input_ok && !wayland {
        notes.push("Input simulation failed to initialize (needs X11).".to_string());
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
            input_permission_granted: true,
            screen_capture_exclusion: false,
            notes,
        },
        input_permission_granted: super::always_granted,
    }
}
