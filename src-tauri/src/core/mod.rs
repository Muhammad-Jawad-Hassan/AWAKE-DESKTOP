//! Platform-independent application core: sessions, timers, inactivity,
//! activity scheduling, profiles and configuration.

pub mod activity;
pub mod config;
pub mod history;
pub mod inactivity;
pub mod ports;
pub mod profiles;
pub mod serde_secs;
pub mod session;
pub mod stats;
pub mod templates;
pub mod timer;

pub use activity::{enabled_activities, ActivityKind};
pub use config::{AppConfig, AppSettings};
pub use history::HistoryEntry;
pub use inactivity::InactivityMonitor;
pub use ports::{PlatformCapabilities, PlatformError, PowerLease, WindowHandle};
pub use profiles::ActivityProfile;
pub use session::{Session, SessionConfig, SessionError, SessionState};
pub use stats::SessionStats;
pub use templates::SessionTemplate;
