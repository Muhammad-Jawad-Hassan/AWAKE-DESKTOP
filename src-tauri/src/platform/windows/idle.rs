//! Windows idle detection via `GetLastInputInfo`.

use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

use crate::core::ports::{IdleProvider, PlatformError};

pub struct WindowsIdleProvider;

impl IdleProvider for WindowsIdleProvider {
    fn idle_seconds(&self) -> Result<f64, PlatformError> {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let ok = unsafe { GetLastInputInfo(&mut info) };
        if ok.as_bool() {
            let now = unsafe { GetTickCount() };
            let idle_ms = now.wrapping_sub(info.dwTime);
            Ok(idle_ms as f64 / 1000.0)
        } else {
            Err(PlatformError::OperationFailed(
                "GetLastInputInfo failed".into(),
            ))
        }
    }
}
