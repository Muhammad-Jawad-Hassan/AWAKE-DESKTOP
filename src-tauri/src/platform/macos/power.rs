//! macOS power management via the `caffeinate` system utility.

use std::process::{Child, Command};
use std::time::Duration;

use crate::core::ports::{PlatformError, PowerLease, PowerManager};

const EXPIRY_MARGIN: Duration = Duration::from_secs(30);

pub struct MacOsPowerManager;

impl PowerManager for MacOsPowerManager {
    fn acquire(
        &self,
        keep_system_awake: bool,
        keep_display_awake: bool,
        max_duration: Duration,
    ) -> Result<Box<dyn PowerLease>, PlatformError> {
        let mut command = Command::new("/usr/bin/caffeinate");
        if keep_system_awake {
            command.arg("-i");
        }
        if keep_display_awake {
            command.arg("-d");
        }
        command
            .arg("-t")
            .arg((max_duration + EXPIRY_MARGIN).as_secs().to_string());
        let child = command.spawn().map_err(|e| {
            PlatformError::OperationFailed(format!("failed to launch caffeinate: {e}"))
        })?;
        Ok(Box::new(CaffeinateLease { child }))
    }
}

struct CaffeinateLease {
    child: Child,
}

impl PowerLease for CaffeinateLease {
    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for CaffeinateLease {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
