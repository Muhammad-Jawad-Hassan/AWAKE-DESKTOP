//! Platform-independent application core: sessions, timers, inactivity,
//! activity scheduling, profiles and configuration.

pub mod activity;
pub mod config;
pub mod history;
pub mod inactivity;
pub mod ports;
pub mod profiles;
pub mod session;
pub mod stats;
pub mod timer;

pub use activity::{
    decide_outcome, enabled_activities, next_delay, split_path_steps, ActivityKind, ActivityOutcome,
};
pub use config::{AppConfig, AppSettings};
pub use history::HistoryEntry;
pub use inactivity::InactivityMonitor;
pub use ports::{PlatformCapabilities, PlatformError, PowerLease, WindowHandle};
pub use profiles::{ActivityProfile, ClickKind, MouseButton};
pub use session::{Session, SessionConfig, SessionError, SessionState};
pub use stats::SessionStats;
