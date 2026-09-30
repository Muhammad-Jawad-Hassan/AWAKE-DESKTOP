//! Activity profiles: named, reusable bundles of activity-automation configuration.

use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Keys that do nothing on their own, so a tap is safe in any focused app.
pub const SAFE_AUTOMATION_KEYS: [&str; 2] = ["Shift", "Control"];

/// Key choice that taps a different safe key each time.
pub const RANDOM_KEY: &str = "Random";

/// A lone Shift can switch CJK input modes; a lone Control doesn't.
pub const DEFAULT_KEY: &str = "Control";

/// Most automated actions of all kinds per minute; one activity may use all of it.
pub const MAX_SAFETY_CAP: u32 = 60;
pub const MAX_PER_MINUTE: u32 = MAX_SAFETY_CAP;
pub const MAX_CORNER_MARGIN_PX: u32 = 200;

/// The canonical spelling of a valid key choice, or `None` if it isn't one.
fn canonical_key_choice(key: &str) -> Option<&'static str> {
    SAFE_AUTOMATION_KEYS
        .into_iter()
        .chain([RANDOM_KEY])
        .find(|choice| choice.eq_ignore_ascii_case(key.trim()))
}

fn is_valid_key_choice(key: &str) -> bool {
    canonical_key_choice(key).is_some()
}

fn default_per_minute() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "LegacyMouseConfig")]
pub struct MouseConfig {
    pub enabled: bool,
    #[serde(rename = "perMinute")]
    pub per_minute: u32,
}

/// Reads profiles saved while clicks existed; `movement: false` meant clicks only.
#[derive(Deserialize)]
struct LegacyMouseConfig {
    enabled: bool,
    #[serde(default = "enabled_by_default")]
    movement: bool,
    #[serde(rename = "perMinute", default = "default_per_minute")]
    per_minute: u32,
}

fn enabled_by_default() -> bool {
    true
}

impl From<LegacyMouseConfig> for MouseConfig {
    fn from(legacy: LegacyMouseConfig) -> Self {
        Self {
            enabled: legacy.enabled && legacy.movement,
            per_minute: legacy.per_minute,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyboardConfig {
    pub enabled: bool,
    /// One of `SAFE_AUTOMATION_KEYS` or `RANDOM_KEY`.
    pub key: String,
    #[serde(rename = "perMinute", default = "default_per_minute")]
    pub per_minute: u32,
}

impl KeyboardConfig {
    /// The key to tap next; resolves `RANDOM_KEY` to a random safe key.
    pub fn next_key(&self, rng: &mut impl Rng) -> &str {
        if self.key.trim().eq_ignore_ascii_case(RANDOM_KEY) {
            SAFE_AUTOMATION_KEYS[rng.gen_range(0..SAFE_AUTOMATION_KEYS.len())]
        } else {
            self.key.trim()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GestureConfig {
    pub horizontal: bool,
    pub vertical: bool,
    /// Shared by both axes when both are on.
    #[serde(rename = "perMinute", default = "default_per_minute")]
    pub per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SafetySettings {
    /// Skip mouse moves that start or end this close to a screen corner.
    pub avoid_screen_corners_px: u32,
    /// Most automated actions of all kinds in any 60 seconds.
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
    #[serde(with = "super::serde_secs")]
    pub inactivity_threshold: Duration,
    pub mouse: MouseConfig,
    pub keyboard: KeyboardConfig,
    pub gestures: GestureConfig,
    pub safety: SafetySettings,
    /// Built-in profiles ship with the app and can't be changed or deleted.
    pub built_in: bool,
}

impl ActivityProfile {
    /// A new custom profile with default settings.
    pub fn new_custom(id: String) -> Self {
        Self {
            id,
            name: "New profile".to_string(),
            inactivity_threshold: Duration::from_secs(5 * 60),
            mouse: MouseConfig {
                enabled: true,
                per_minute: 1,
            },
            keyboard: KeyboardConfig {
                enabled: false,
                key: DEFAULT_KEY.to_string(),
                per_minute: 1,
            },
            gestures: GestureConfig {
                horizontal: false,
                vertical: false,
                per_minute: 1,
            },
            safety: SafetySettings::default(),
            built_in: false,
        }
    }

    fn scrolls(&self) -> bool {
        self.gestures.horizontal || self.gestures.vertical
    }

    fn enabled_rates(&self) -> impl Iterator<Item = u32> {
        [
            (self.mouse.enabled, self.mouse.per_minute),
            (self.keyboard.enabled, self.keyboard.per_minute),
            (self.scrolls(), self.gestures.per_minute),
        ]
        .into_iter()
        .filter_map(|(on, rate)| on.then_some(rate))
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.name.trim().is_empty() {
            return Err(ProfileError::EmptyName);
        }
        if self.inactivity_threshold.is_zero() {
            return Err(ProfileError::InactivityThresholdTooShort);
        }
        if self.enabled_rates().next().is_none() {
            return Err(ProfileError::NoActivityEnabled);
        }
        if self
            .enabled_rates()
            .any(|rate| !(1..=MAX_PER_MINUTE).contains(&rate))
        {
            return Err(ProfileError::RateOutOfRange);
        }
        if !(1..=MAX_SAFETY_CAP).contains(&self.safety.max_actions_per_minute) {
            return Err(ProfileError::SafetyCapOutOfRange);
        }
        if self.enabled_rates().sum::<u32>() > self.safety.max_actions_per_minute {
            return Err(ProfileError::RatesExceedSafetyCap);
        }
        if self.safety.avoid_screen_corners_px > MAX_CORNER_MARGIN_PX {
            return Err(ProfileError::CornerMarginTooLarge);
        }
        if !is_valid_key_choice(&self.keyboard.key) {
            return Err(ProfileError::UnsafeKeyboardKey);
        }
        Ok(())
    }

    /// Repairs data saved by older versions; `validate` reports anything left.
    pub fn migrate(&mut self) {
        self.keyboard.key = canonical_key_choice(&self.keyboard.key)
            .unwrap_or(DEFAULT_KEY)
            .to_string();
        for rate in [
            &mut self.mouse.per_minute,
            &mut self.keyboard.per_minute,
            &mut self.gestures.per_minute,
        ] {
            *rate = (*rate).clamp(1, MAX_PER_MINUTE);
        }
        let total = self.enabled_rates().sum::<u32>();
        let cap = &mut self.safety.max_actions_per_minute;
        *cap = (*cap).max(total).clamp(1, MAX_SAFETY_CAP);
        let margin = &mut self.safety.avoid_screen_corners_px;
        *margin = (*margin).min(MAX_CORNER_MARGIN_PX);
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum ProfileError {
    #[error("profile name must not be empty")]
    EmptyName,
    #[error("inactivity threshold must be greater than zero")]
    InactivityThresholdTooShort,
    #[error("at least one activity type must be enabled")]
    NoActivityEnabled,
    #[error("keyboard automation only supports {} or {RANDOM_KEY}", SAFE_AUTOMATION_KEYS.join(", "))]
    UnsafeKeyboardKey,
    #[error("each activity must run between 1 and {MAX_PER_MINUTE} times per minute")]
    RateOutOfRange,
    #[error("the safety cap must be between 1 and {MAX_SAFETY_CAP} actions per minute")]
    SafetyCapOutOfRange,
    #[error("combined per-minute rates exceed the safety cap")]
    RatesExceedSafetyCap,
    #[error("the screen-corner margin must be at most {MAX_CORNER_MARGIN_PX}px")]
    CornerMarginTooLarge,
}

/// Limits the UI needs to build a valid profile or session.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub safe_keys: &'static [&'static str],
    pub random_key: &'static str,
    pub max_per_minute: u32,
    pub max_safety_cap: u32,
    pub max_session_secs: u64,
}

pub fn limits() -> Limits {
    Limits {
        safe_keys: &SAFE_AUTOMATION_KEYS,
        random_key: RANDOM_KEY,
        max_per_minute: MAX_PER_MINUTE,
        max_safety_cap: MAX_SAFETY_CAP,
        max_session_secs: super::timer::MAX_SESSION_DURATION.as_secs(),
    }
}

/// The three profiles that ship built in.
pub fn built_in_profiles() -> Vec<ActivityProfile> {
    let base = |id: &str, name: &str, threshold_mins: u64| ActivityProfile {
        name: name.to_string(),
        inactivity_threshold: Duration::from_secs(threshold_mins * 60),
        built_in: true,
        ..ActivityProfile::new_custom(id.to_string())
    };

    let developer = base("developer", "Developer", 5);

    let mut presentation = base("presentation", "Presentation", 2);
    presentation.safety.max_actions_per_minute = 3;

    let mut testing = base("testing", "Testing", 1);
    testing.mouse.per_minute = 2;
    testing.keyboard = KeyboardConfig {
        enabled: true,
        per_minute: 2,
        ..testing.keyboard
    };
    testing.gestures = GestureConfig {
        horizontal: true,
        vertical: true,
        per_minute: 2,
    };

    vec![developer, presentation, testing]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom() -> ActivityProfile {
        ActivityProfile::new_custom("custom-1".to_string())
    }

    #[test]
    fn built_in_and_new_profiles_are_valid() {
        for profile in built_in_profiles() {
            assert!(
                profile.validate().is_ok(),
                "{} failed validation",
                profile.name
            );
            assert!(profile.built_in);
        }
        assert!(custom().validate().is_ok());
    }

    #[test]
    fn rejects_zero_inactivity_threshold() {
        let mut profile = custom();
        profile.inactivity_threshold = Duration::ZERO;
        assert_eq!(
            profile.validate(),
            Err(ProfileError::InactivityThresholdTooShort)
        );
    }

    #[test]
    fn rejects_unsafe_keyboard_keys_even_when_keyboard_is_off() {
        let mut profile = custom();
        for key in [
            "A", "Enter", "Tab", "Alt", "Meta", "F5", "F13", "F15", "Space", " ",
        ] {
            profile.keyboard.key = key.to_string();
            assert_eq!(
                profile.validate(),
                Err(ProfileError::UnsafeKeyboardKey),
                "{key:?} should be rejected"
            );
        }
    }

    #[test]
    fn accepts_safe_keys_and_random_in_any_case() {
        let mut profile = custom();
        profile.keyboard.enabled = true;
        for key in ["Shift", "control", "SHIFT", "random"] {
            profile.keyboard.key = key.to_string();
            assert!(profile.validate().is_ok(), "{key} should be accepted");
        }
    }

    #[test]
    fn random_key_only_ever_taps_safe_keys_and_varies() {
        use rand::{rngs::StdRng, SeedableRng};
        let keyboard = KeyboardConfig {
            key: RANDOM_KEY.to_string(),
            ..custom().keyboard
        };
        let mut rng = StdRng::seed_from_u64(3);
        let picks: std::collections::HashSet<_> = (0..200)
            .map(|_| keyboard.next_key(&mut rng).to_string())
            .collect();
        assert!(picks
            .iter()
            .all(|key| SAFE_AUTOMATION_KEYS.contains(&key.as_str())));
        assert_eq!(picks.len(), SAFE_AUTOMATION_KEYS.len());
    }

    #[test]
    fn rejects_a_profile_with_nothing_enabled() {
        let mut profile = custom();
        profile.mouse.enabled = false;
        assert_eq!(profile.validate(), Err(ProfileError::NoActivityEnabled));
    }

    #[test]
    fn rejects_rates_outside_the_allowed_range() {
        let mut profile = custom();
        for rate in [0, MAX_PER_MINUTE + 1] {
            profile.mouse.per_minute = rate;
            assert_eq!(profile.validate(), Err(ProfileError::RateOutOfRange));
        }
    }

    #[test]
    fn ignores_the_rate_of_a_disabled_activity() {
        let mut profile = custom();
        profile.keyboard.per_minute = 0;
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn rejects_rates_that_add_up_past_the_safety_cap() {
        let mut profile = custom();
        profile.safety.max_actions_per_minute = 6;
        profile.mouse.per_minute = 4;
        profile.keyboard.enabled = true;
        profile.keyboard.per_minute = 2;
        assert!(profile.validate().is_ok());
        profile.keyboard.per_minute = 3;
        assert_eq!(profile.validate(), Err(ProfileError::RatesExceedSafetyCap));
    }

    #[test]
    fn rejects_out_of_range_safety_settings() {
        let mut profile = custom();
        profile.safety.max_actions_per_minute = MAX_SAFETY_CAP + 1;
        assert_eq!(profile.validate(), Err(ProfileError::SafetyCapOutOfRange));

        let mut profile = custom();
        profile.safety.avoid_screen_corners_px = MAX_CORNER_MARGIN_PX + 1;
        assert_eq!(profile.validate(), Err(ProfileError::CornerMarginTooLarge));
    }

    #[test]
    fn click_only_legacy_profiles_load_with_the_mouse_off() {
        let mut value = serde_json::to_value(custom()).unwrap();
        value["mouse"]["movement"] = false.into();
        value["mouse"]["button"] = "left".into();
        let restored: ActivityProfile = serde_json::from_value(value).unwrap();
        assert!(!restored.mouse.enabled);
    }

    #[test]
    fn legacy_profiles_load_ignoring_removed_fields() {
        let mut value = serde_json::to_value(custom()).unwrap();
        value["minDelay"] = 30.into();
        value["mouse"]["movement"] = true.into();
        value["mouse"]["randomize"] = true.into();
        value["keyboard"]["modifiers"] = serde_json::json!(["Meta"]);
        value["gestures"]["custom"] = "x".into();
        for section in ["keyboard", "gestures"] {
            value[section].as_object_mut().unwrap().remove("perMinute");
        }
        let restored: ActivityProfile = serde_json::from_value(value).unwrap();
        assert_eq!(restored, custom());
    }

    #[test]
    fn migrate_repairs_legacy_keys_rates_and_caps() {
        let mut profile = custom();
        profile.keyboard = KeyboardConfig {
            enabled: false,
            key: "A".to_string(),
            per_minute: 0,
        };
        profile.mouse.per_minute = 99;
        profile.safety = SafetySettings {
            avoid_screen_corners_px: u32::MAX,
            max_actions_per_minute: 2,
        };

        profile.migrate();

        assert_eq!(profile.keyboard.key, DEFAULT_KEY);
        assert_eq!(profile.keyboard.per_minute, 1);
        assert_eq!(profile.mouse.per_minute, MAX_PER_MINUTE);
        assert_eq!(profile.safety.max_actions_per_minute, MAX_SAFETY_CAP);
        assert_eq!(profile.safety.avoid_screen_corners_px, MAX_CORNER_MARGIN_PX);
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn migrate_normalizes_key_case_and_drops_removed_keys() {
        let mut profile = custom();
        profile.keyboard.key = "shift".to_string();
        profile.migrate();
        assert_eq!(profile.keyboard.key, "Shift");

        profile.keyboard.key = "F15".to_string();
        profile.migrate();
        assert_eq!(profile.keyboard.key, DEFAULT_KEY);
    }

    #[test]
    fn migrate_leaves_valid_profiles_untouched() {
        for mut profile in built_in_profiles() {
            let before = profile.clone();
            profile.migrate();
            assert_eq!(profile, before);
        }
    }

    #[test]
    fn error_messages_are_built_from_the_limits() {
        assert_eq!(
            ProfileError::UnsafeKeyboardKey.to_string(),
            "keyboard automation only supports Shift, Control or Random"
        );
        assert!(ProfileError::RateOutOfRange.to_string().contains("60"));
        assert!(ProfileError::SafetyCapOutOfRange.to_string().contains("60"));
    }
}
