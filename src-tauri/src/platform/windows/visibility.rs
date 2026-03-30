//! Excludes the app window from screen capture via `SetWindowDisplayAffinity`.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};

use crate::core::ports::{PlatformError, VisibilityManager, WindowHandle};

pub struct WindowsVisibilityManager;

impl VisibilityManager for WindowsVisibilityManager {
    fn set_exclude_from_capture(
        &self,
        window: WindowHandle,
        exclude: bool,
    ) -> Result<(), PlatformError> {
        let WindowHandle::Windows(raw) = window;
        let hwnd = HWND(raw as *mut std::ffi::c_void);
        let affinity = if exclude {
            WDA_EXCLUDEFROMCAPTURE
        } else {
            WDA_NONE
        };
        unsafe { SetWindowDisplayAffinity(hwnd, affinity) }
            .map_err(|e| PlatformError::OperationFailed(e.to_string()))
    }
}
