//! Excludes the app window from screen capture via `NSWindow.sharingType`.

use objc::runtime::Object;
use objc::{msg_send, sel, sel_impl};

use crate::core::ports::{PlatformError, VisibilityManager, WindowHandle};

const NS_WINDOW_SHARING_NONE: i64 = 0;
const NS_WINDOW_SHARING_READ_WRITE: i64 = 2;

pub struct MacOsVisibilityManager;

impl VisibilityManager for MacOsVisibilityManager {
    fn set_exclude_from_capture(
        &self,
        window: WindowHandle,
        exclude: bool,
    ) -> Result<(), PlatformError> {
        let WindowHandle::MacOs(ptr) = window;
        if ptr.is_null() {
            return Err(PlatformError::OperationFailed("null NSView pointer".into()));
        }
        let sharing_type = if exclude {
            NS_WINDOW_SHARING_NONE
        } else {
            NS_WINDOW_SHARING_READ_WRITE
        };
        unsafe {
            let ns_view = ptr as *mut Object;
            let ns_window: *mut Object = msg_send![ns_view, window];
            if ns_window.is_null() {
                return Err(PlatformError::OperationFailed(
                    "NSView has no NSWindow".into(),
                ));
            }
            let _: () = msg_send![ns_window, setSharingType: sharing_type];
        }
        Ok(())
    }
}
