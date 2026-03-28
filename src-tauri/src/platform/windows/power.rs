//! Windows power management via `SetThreadExecutionState`, whose assertion is
//! scoped to the calling thread, so a lease owns one thread for its lifetime.

use std::sync::mpsc;
use std::thread::JoinHandle;

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

        let (acquired_tx, acquired_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("awake-power-lease".into())
            .spawn(move || {
                let previous = unsafe { SetThreadExecutionState(flags) };
                if acquired_tx.send(previous.0 != 0).is_err() {
                    return;
                }
                // Block here, on this same thread, until told to release.
                let _ = release_rx.recv();
                unsafe {
                    let _ = SetThreadExecutionState(ES_CONTINUOUS);
                }
            })
            .map_err(|e| {
                PlatformError::OperationFailed(format!("failed to start power-lease thread: {e}"))
            })?;

        let acquired = acquired_rx.recv().unwrap_or(false);
        if !acquired {
            let _ = release_tx.send(());
            let _ = thread.join();
            return Err(PlatformError::OperationFailed(
                "SetThreadExecutionState failed".into(),
            ));
        }

        Ok(Box::new(WindowsPowerLease {
            release_tx: Some(release_tx),
            thread: Some(thread),
        }))
    }
}

struct WindowsPowerLease {
    release_tx: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl PowerLease for WindowsPowerLease {
    fn is_alive(&mut self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }
}

impl Drop for WindowsPowerLease {
    fn drop(&mut self) {
        if let Some(tx) = self.release_tx.take() {
            let _ = tx.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
