//! Platform abstraction layer: the only place that selects
//! a concrete `core::ports` implementation via `cfg(target_os)`.

mod input;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use std::sync::Arc;

use crate::core::ports::{
    IdleProvider, InputSimulator, PlatformCapabilities, PowerManager, VisibilityManager,
};

/// The assembled platform capabilities for the OS this binary targets.
#[derive(Clone)]
pub struct Platform {
    pub power: Arc<dyn PowerManager>,
    pub idle: Arc<dyn IdleProvider>,
    pub input: Arc<dyn InputSimulator>,
    pub visibility: Arc<dyn VisibilityManager>,
    pub capabilities: PlatformCapabilities,
}

/// Builds the platform implementation for the OS this binary was compiled for.
pub fn current() -> Platform {
    #[cfg(target_os = "macos")]
    {
        macos::build()
    }
    #[cfg(target_os = "windows")]
    {
        windows::build()
    }
    #[cfg(target_os = "linux")]
    {
        linux::build()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        compile_error!("Awake only supports macOS, Windows and Linux");
    }
}
