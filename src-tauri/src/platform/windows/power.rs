//! Windows power management via `SetThreadExecutionState`.

use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

use crate::core::ports::{PlatformError, PowerLease, PowerManager};

pub struct WindowsPowerManager;

impl PowerManager for WindowsPowerManager {
    fn acquire(
        &self,
        keep_system_awake: bool,
        keep_display_awake: bool,
        _max_duration: std::time::Duration,
    ) -> Result<Box<dyn PowerLease>, PlatformError> {
        let mut flags = ES_CONTINUOUS;
        if keep_system_awake {
            flags |= ES_SYSTEM_REQUIRED;
        }
        if keep_display_awake {
            flags |= ES_DISPLAY_REQUIRED;
        }

        let previous = unsafe { SetThreadExecutionState(flags) };
        if previous.0 == 0 {
            return Err(PlatformError::OperationFailed(
                "SetThreadExecutionState failed".into(),
            ));
        }
        Ok(Box::new(WindowsPowerLease))
    }
}

struct WindowsPowerLease;

impl PowerLease for WindowsPowerLease {}

impl Drop for WindowsPowerLease {
    fn drop(&mut self) {
        unsafe {
            let _ = SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}
