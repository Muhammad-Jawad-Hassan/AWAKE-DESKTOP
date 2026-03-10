//! Platform-independent application core: sessions, timers, inactivity,
//! activity scheduling, profiles and configuration.

pub mod profiles;
pub mod session;
pub mod timer;

pub use profiles::{ActivityProfile, ClickKind, MouseButton};
pub use session::{Session, SessionConfig, SessionError, SessionState};
