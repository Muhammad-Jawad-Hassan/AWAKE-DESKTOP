//! No general screen-capture exclusion mechanism exists across Linux compositors.

use crate::core::ports::{PlatformError, VisibilityManager, WindowHandle};

pub struct LinuxVisibilityManager;

impl VisibilityManager for LinuxVisibilityManager {
    fn set_exclude_from_capture(
        &self,
        _window: WindowHandle,
        _exclude: bool,
    ) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "screen-capture exclusion is not supported on Linux".into(),
        ))
    }
}
