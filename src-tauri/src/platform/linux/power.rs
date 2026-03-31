//! Linux power management via `systemd-inhibit`, where available.

use std::process::{Child, Command};
use std::time::Duration;

use crate::core::ports::{PlatformError, PowerLease, PowerManager};

pub struct LinuxPowerManager;

impl PowerManager for LinuxPowerManager {
    fn acquire(
        &self,
        _keep_system_awake: bool,
        _keep_display_awake: bool,
        max_duration: Duration,
    ) -> Result<Box<dyn PowerLease>, PlatformError> {
        if which::which("systemd-inhibit").is_err() {
            return Err(PlatformError::Unsupported(
                "systemd-inhibit not found".into(),
            ));
        }
        let sleep_secs = max_duration.as_secs().to_string();
        let child = Command::new("systemd-inhibit")
            .args([
                "--what=idle:sleep",
                "--who=Awake",
                "--why=Awake session active",
                "--mode=block",
                "sleep",
                &sleep_secs,
            ])
            .spawn()
            .map_err(|e| {
                PlatformError::OperationFailed(format!("failed to launch systemd-inhibit: {e}"))
            })?;
        Ok(Box::new(InhibitLease { child }))
    }
}

struct InhibitLease {
    child: Child,
}

impl PowerLease for InhibitLease {
    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for InhibitLease {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
