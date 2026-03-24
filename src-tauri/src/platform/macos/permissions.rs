//! Accessibility permission check/request, required for input simulation.

use std::process::Command;

use crate::core::ports::PlatformError;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn is_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Opens System Settings to the Accessibility pane for the user to grant access.
pub fn open_accessibility_settings() -> Result<(), PlatformError> {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map(|_| ())
        .map_err(|e| PlatformError::OperationFailed(e.to_string()))
}
