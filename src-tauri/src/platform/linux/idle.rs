//! Linux idle detection via `xprintidle` (X11 only; unsupported on Wayland).

use std::process::Command;

use crate::core::ports::{IdleProvider, PlatformError};

pub struct LinuxIdleProvider;

impl IdleProvider for LinuxIdleProvider {
    fn idle_seconds(&self) -> Result<f64, PlatformError> {
        if which::which("xprintidle").is_err() {
            return Err(PlatformError::Unsupported(
                "xprintidle not found (idle detection needs X11)".into(),
            ));
        }
        let output = Command::new("xprintidle").output().map_err(|e| {
            PlatformError::OperationFailed(format!("failed to run xprintidle: {e}"))
        })?;
        let text = String::from_utf8_lossy(&output.stdout);
        let idle_ms: u64 = text
            .trim()
            .parse()
            .map_err(|_| PlatformError::OperationFailed("unexpected xprintidle output".into()))?;
        Ok(idle_ms as f64 / 1000.0)
    }
}
