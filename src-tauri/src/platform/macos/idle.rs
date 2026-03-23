//! macOS idle detection via CoreGraphics; needs no special permission.

use crate::core::ports::{IdleProvider, PlatformError};

const K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: i32 = 1;
const K_CG_ANY_INPUT_EVENT_TYPE: u32 = u32::MAX;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceSecondsSinceLastEventType(state_id: i32, event_type: u32) -> f64;
}

pub struct MacOsIdleProvider;

impl IdleProvider for MacOsIdleProvider {
    fn idle_seconds(&self) -> Result<f64, PlatformError> {
        let seconds = unsafe {
            CGEventSourceSecondsSinceLastEventType(
                K_CG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE,
                K_CG_ANY_INPUT_EVENT_TYPE,
            )
        };
        if seconds.is_finite() && seconds >= 0.0 {
            Ok(seconds)
        } else {
            Err(PlatformError::OperationFailed(
                "CoreGraphics returned an invalid idle time".into(),
            ))
        }
    }
}
