//! The `Session` state machine: the central abstraction of the application.
//! Everything attaches to a session and knows nothing about *why* it exists.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::timer::{validate_duration, DurationError, SessionTimer};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    #[serde(with = "duration_secs")]
    pub duration: Duration,
    pub keep_system_awake: bool,
    pub keep_display_awake: bool,
    #[serde(with = "duration_secs")]
    pub inactivity_threshold: Duration,
    /// `None` means activity automation is disabled for this session.
    pub activity_profile_id: Option<String>,
}

impl SessionConfig {
    pub fn validate(&self) -> Result<(), SessionError> {
        validate_duration(self.duration).map_err(SessionError::InvalidDuration)?;
        if self.inactivity_threshold.is_zero() {
            return Err(SessionError::InvalidInactivityThreshold);
        }
        if !self.keep_system_awake && !self.keep_display_awake && self.activity_profile_id.is_none()
        {
            return Err(SessionError::NothingToDo);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Active,
    /// The configured duration elapsed naturally.
    Completed,
    /// The user stopped the session before it expired.
    Stopped,
    /// A platform operation failed badly enough to abort the session.
    Failed,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum SessionError {
    #[error("invalid duration: {0}")]
    InvalidDuration(#[from] DurationError),
    #[error("inactivity threshold must be greater than zero")]
    InvalidInactivityThreshold,
    #[error("session must keep the system or display awake, or run an activity profile")]
    NothingToDo,
    #[error("a session is already active")]
    AlreadyActive,
    #[error("no session is currently active")]
    NotActive,
}

#[derive(Debug, Clone)]
pub struct Session {
    config: SessionConfig,
    state: SessionState,
    timer: Option<SessionTimer>,
}

impl Session {
    pub fn idle(config: SessionConfig) -> Self {
        Self {
            config,
            state: SessionState::Idle,
            timer: None,
        }
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn timer(&self) -> Option<&SessionTimer> {
        self.timer.as_ref()
    }

    pub fn start(&mut self, now: Instant) -> Result<(), SessionError> {
        if self.state == SessionState::Active {
            return Err(SessionError::AlreadyActive);
        }
        self.config.validate()?;
        self.timer = Some(SessionTimer::new(now, self.config.duration));
        self.state = SessionState::Active;
        Ok(())
    }

    /// Adds time to a running session, capped at the same 24h sanity bound as a fresh session.
    pub fn extend(&mut self, extra: Duration) -> Result<(), SessionError> {
        if self.state != SessionState::Active {
            return Err(SessionError::NotActive);
        }
        let new_duration = self.config.duration + extra;
        validate_duration(new_duration).map_err(SessionError::InvalidDuration)?;
        self.config.duration = new_duration;
        if let Some(timer) = self.timer.as_mut() {
            timer.extend(extra);
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Active {
            return Err(SessionError::NotActive);
        }
        self.state = SessionState::Stopped;
        Ok(())
    }

    pub fn fail(&mut self) {
        self.state = SessionState::Failed;
    }

    /// Transitions to `Completed` if the duration has elapsed; no-op otherwise.
    pub fn tick(&mut self, now: Instant) -> SessionState {
        if self.state == SessionState::Active {
            if let Some(timer) = &self.timer {
                if timer.is_expired(now) {
                    self.state = SessionState::Completed;
                }
            }
        }
        self.state
    }

    pub fn remaining(&self, now: Instant) -> Duration {
        match &self.timer {
            Some(timer) if self.state == SessionState::Active => timer.remaining(now),
            _ => Duration::ZERO,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active
    }
}

mod duration_secs {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> SessionConfig {
        SessionConfig {
            duration: Duration::from_secs(60 * 60),
            keep_system_awake: true,
            keep_display_awake: false,
            inactivity_threshold: Duration::from_secs(300),
            activity_profile_id: None,
        }
    }

    #[test]
    fn starts_from_idle() {
        let mut session = Session::idle(valid_config());
        assert_eq!(session.state(), SessionState::Idle);
        session.start(Instant::now()).unwrap();
        assert_eq!(session.state(), SessionState::Active);
    }

    #[test]
    fn cannot_start_twice() {
        let mut session = Session::idle(valid_config());
        session.start(Instant::now()).unwrap();
        assert_eq!(
            session.start(Instant::now()),
            Err(SessionError::AlreadyActive)
        );
    }

    #[test]
    fn rejects_config_with_nothing_to_do() {
        let config = SessionConfig {
            keep_system_awake: false,
            keep_display_awake: false,
            activity_profile_id: None,
            ..valid_config()
        };
        let mut session = Session::idle(config);
        assert_eq!(
            session.start(Instant::now()),
            Err(SessionError::NothingToDo)
        );
    }

    #[test]
    fn stop_transitions_active_to_stopped() {
        let mut session = Session::idle(valid_config());
        session.start(Instant::now()).unwrap();
        session.stop().unwrap();
        assert_eq!(session.state(), SessionState::Stopped);
    }

    #[test]
    fn cannot_stop_a_session_that_is_not_active() {
        let mut session = Session::idle(valid_config());
        assert_eq!(session.stop(), Err(SessionError::NotActive));
    }

    #[test]
    fn tick_completes_session_after_duration_elapses() {
        let config = SessionConfig {
            duration: Duration::from_secs(10),
            ..valid_config()
        };
        let mut session = Session::idle(config);
        let start = Instant::now();
        session.start(start).unwrap();

        session.tick(start + Duration::from_secs(5));
        assert_eq!(session.state(), SessionState::Active);

        session.tick(start + Duration::from_secs(10));
        assert_eq!(session.state(), SessionState::Completed);
    }

    #[test]
    fn remaining_is_zero_when_not_active() {
        let session = Session::idle(valid_config());
        assert_eq!(session.remaining(Instant::now()), Duration::ZERO);
    }

    #[test]
    fn stopped_session_cannot_be_stopped_again() {
        let mut session = Session::idle(valid_config());
        session.start(Instant::now()).unwrap();
        session.stop().unwrap();
        assert_eq!(session.stop(), Err(SessionError::NotActive));
    }

    #[test]
    fn extend_pushes_remaining_time_and_duration_out() {
        let config = SessionConfig {
            duration: Duration::from_secs(10),
            ..valid_config()
        };
        let mut session = Session::idle(config);
        let start = Instant::now();
        session.start(start).unwrap();

        session.extend(Duration::from_secs(20)).unwrap();

        assert_eq!(session.remaining(start), Duration::from_secs(30));
        assert_eq!(session.config().duration, Duration::from_secs(30));
    }

    #[test]
    fn cannot_extend_a_session_that_is_not_active() {
        let mut session = Session::idle(valid_config());
        assert_eq!(
            session.extend(Duration::from_secs(10)),
            Err(SessionError::NotActive)
        );
    }

    #[test]
    fn extend_rejects_pushing_past_the_24h_cap() {
        let config = SessionConfig {
            duration: Duration::from_secs(23 * 60 * 60),
            ..valid_config()
        };
        let mut session = Session::idle(config);
        session.start(Instant::now()).unwrap();
        assert!(session.extend(Duration::from_secs(2 * 60 * 60)).is_err());
    }
}
