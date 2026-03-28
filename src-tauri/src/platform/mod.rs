//! Platform abstraction layer: the only place that selects
//! a concrete `core::ports` implementation via `cfg(target_os)`.

mod input;

#[cfg(target_os = "macos")]
pub mod macos;

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
    #[cfg(not(target_os = "macos"))]
    {
        compile_error!("Awake only supports macOS so far");
    }
}
