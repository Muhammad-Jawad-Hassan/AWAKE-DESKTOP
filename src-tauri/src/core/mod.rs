//! Platform-independent application core: sessions, timers, inactivity,
//! activity scheduling, profiles and configuration.

pub mod session;
pub mod timer;

pub use session::{Session, SessionConfig, SessionError, SessionState};
