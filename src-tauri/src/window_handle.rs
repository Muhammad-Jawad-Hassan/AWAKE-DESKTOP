//! Converts a Tauri window into the opaque `core::WindowHandle` the platform layer expects.

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri::WebviewWindow;

use crate::core::{PlatformError, WindowHandle};

pub fn native_handle(window: &WebviewWindow) -> Result<WindowHandle, PlatformError> {
    let handle = window
        .window_handle()
        .map_err(|e| PlatformError::OperationFailed(e.to_string()))?;
    match handle.as_raw() {
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(h) => Ok(WindowHandle::MacOs(h.ns_view.as_ptr())),
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(h) => Ok(WindowHandle::Windows(h.hwnd.get())),
        #[cfg(target_os = "linux")]
        RawWindowHandle::Xlib(h) => Ok(WindowHandle::Linux(h.window)),
        _ => Err(PlatformError::Unsupported(
            "unrecognized window handle type".into(),
        )),
    }
}
