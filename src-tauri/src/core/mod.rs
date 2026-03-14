//! Platform-independent application core: sessions, timers, inactivity,
//! activity scheduling, profiles and configuration.

pub mod activity;
pub mod profiles;
pub mod session;
pub mod timer;

pub use activity::{
    decide_outcome, enabled_activities, next_delay, split_path_steps, ActivityKind, ActivityOutcome,
};
pub use profiles::{ActivityProfile, ClickKind, MouseButton};
pub use session::{Session, SessionConfig, SessionError, SessionState};
