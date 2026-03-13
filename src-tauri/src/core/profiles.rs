//! Activity profiles: named, reusable bundles of activity-automation
//! configuration.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClickKind {
    Single,
    Double,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MouseConfig {
    pub enabled: bool,
    pub movement: bool,
    pub button: MouseButton,
    pub click: ClickKind,
    pub randomize: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            movement: true,
            button: MouseButton::Left,
            click: ClickKind::Single,
            randomize: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyboardConfig {
    pub enabled: bool,
    /// A single, safe key such as "Shift" or "F15" (never a shortcut).
    pub key: String,
    pub modifiers: Vec<String>,
    pub randomize: bool,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            key: "Shift".to_string(),
            modifiers: vec![],
            randomize: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GestureConfig {
    pub horizontal: bool,
    pub vertical: bool,
    pub custom: Option<String>,
}

/// Guardrails the activity engine always enforces, regardless of profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SafetySettings {
    /// Skip automated input within this many pixels of a screen corner.
    pub avoid_screen_corners_px: u32,
    /// Hard cap on automated actions per minute, independent of the delay range.
    pub max_actions_per_minute: u32,
}

impl Default for SafetySettings {
    fn default() -> Self {
        Self {
            avoid_screen_corners_px: 24,
            max_actions_per_minute: 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityProfile {
    pub id: String,
    pub name: String,
    #[serde(with = "duration_secs")]
    pub inactivity_threshold: Duration,
    #[serde(with = "duration_secs")]
    pub min_delay: Duration,
    #[serde(with = "duration_secs")]
    pub max_delay: Duration,
    pub mouse: MouseConfig,
    pub keyboard: KeyboardConfig,
    pub gestures: GestureConfig,
    pub safety: SafetySettings,
    /// Built-in profiles ship with the app and cannot be deleted.
    pub built_in: bool,
}

impl ActivityProfile {
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.name.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if self.inactivity_threshold.is_zero() {
            return Err(ProfileError::InactivityThresholdTooShort);
        }
        if self.min_delay.is_zero() {
            return Err(ProfileError::MinDelayTooShort);
        }
        if self.min_delay > self.max_delay {
            return Err(ProfileError::DelayRangeInverted);
        }
        if !self.mouse.enabled
            && !self.keyboard.enabled
            && !self.gestures.horizontal
            && !self.gestures.vertical
            && self.gestures.custom.is_none()
        {
            return Err(ProfileError::NoActivityEnabled);
        }
        if self.keyboard.enabled && self.keyboard.key.trim().is_empty() {
            return Err(ProfileError::EmptyKeyboardKey);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum ProfileError {
    #[error("profile name must not be empty")]
    EmptyName,
    #[error("inactivity threshold must be greater than zero")]
    InactivityThresholdTooShort,
    #[error("minimum delay must be greater than zero")]
    MinDelayTooShort,
    #[error("minimum delay must not exceed maximum delay")]
    DelayRangeInverted,
    #[error("at least one activity type must be enabled")]
    NoActivityEnabled,
    #[error("keyboard automation is enabled but no key is set")]
    EmptyKeyboardKey,
}

/// The three profiles that ship built in.
pub fn built_in_profiles() -> Vec<ActivityProfile> {
    vec![
        ActivityProfile {
            id: "developer".to_string(),
            name: "Developer".to_string(),
            inactivity_threshold: Duration::from_secs(5 * 60),
            min_delay: Duration::from_secs(30),
            max_delay: Duration::from_secs(180),
            mouse: MouseConfig {
                enabled: true,
                movement: true,
                ..MouseConfig::default()
            },
            keyboard: KeyboardConfig {
                enabled: false,
                ..KeyboardConfig::default()
            },
            gestures: GestureConfig::default(),
            safety: SafetySettings::default(),
            built_in: true,
        },
        ActivityProfile {
            id: "presentation".to_string(),
            name: "Presentation".to_string(),
            inactivity_threshold: Duration::from_secs(2 * 60),
            min_delay: Duration::from_secs(60),
            max_delay: Duration::from_secs(240),
            mouse: MouseConfig {
                enabled: true,
                movement: true,
                randomize: false,
                ..MouseConfig::default()
            },
            keyboard: KeyboardConfig::default(),
            gestures: GestureConfig::default(),
            safety: SafetySettings {
                max_actions_per_minute: 3,
                ..SafetySettings::default()
            },
            built_in: true,
        },
        ActivityProfile {
            id: "testing".to_string(),
            name: "Testing".to_string(),
            inactivity_threshold: Duration::from_secs(60),
            min_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(30),
            mouse: MouseConfig::default(),
            keyboard: KeyboardConfig {
                enabled: true,
                ..KeyboardConfig::default()
            },
            gestures: GestureConfig {
                horizontal: true,
                vertical: true,
                custom: None,
            },
            safety: SafetySettings::default(),
            built_in: true,
        },
    ]
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

    #[test]
    fn built_in_profiles_are_all_valid() {
        for profile in built_in_profiles() {
            assert!(
                profile.validate().is_ok(),
                "{} failed validation",
                profile.name
            );
            assert!(profile.built_in);
        }
    }

    #[test]
    fn rejects_zero_inactivity_threshold() {
        let mut profile = built_in_profiles().remove(0);
        profile.inactivity_threshold = Duration::ZERO;
        assert_eq!(
            profile.validate(),
            Err(ProfileError::InactivityThresholdTooShort)
        );
    }

    #[test]
    fn rejects_enabled_keyboard_with_no_key() {
        let mut profile = built_in_profiles().remove(0);
        profile.keyboard.enabled = true;
        profile.keyboard.key = "  ".to_string();
        assert_eq!(profile.validate(), Err(ProfileError::EmptyKeyboardKey));
    }

    #[test]
    fn rejects_inverted_delay_range() {
        let mut profile = built_in_profiles().remove(0);
        profile.min_delay = Duration::from_secs(200);
        profile.max_delay = Duration::from_secs(100);
        assert_eq!(profile.validate(), Err(ProfileError::DelayRangeInverted));
    }

    #[test]
    fn rejects_profile_with_nothing_enabled() {
        let mut profile = built_in_profiles().remove(0);
        profile.mouse.enabled = false;
        profile.keyboard.enabled = false;
        profile.gestures = GestureConfig::default();
        assert_eq!(profile.validate(), Err(ProfileError::NoActivityEnabled));
    }

    #[test]
    fn round_trips_through_json() {
        let profile = built_in_profiles().remove(0);
        let json = serde_json::to_string(&profile).unwrap();
        let restored: ActivityProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, restored);
    }
}
