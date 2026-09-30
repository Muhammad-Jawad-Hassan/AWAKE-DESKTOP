//! Local configuration persistence: one JSON file in the OS app-config directory.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize, Serializer};

use super::history::HistoryEntry;
use super::profiles::{built_in_profiles, ActivityProfile};
use super::session::SessionConfig;
use super::templates::SessionTemplate;

pub const CONFIG_FILE_NAME: &str = "config.json";

/// Ctrl+Shift+Esc is Task Manager on Windows, so other platforms add Alt.
#[cfg(target_os = "macos")]
const DEFAULT_EMERGENCY_SHORTCUT: &str = "Super+Shift+Escape";
#[cfg(not(target_os = "macos"))]
const DEFAULT_EMERGENCY_SHORTCUT: &str = "Control+Alt+Shift+Escape";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub start_minimized: bool,
    pub launch_at_login: bool,
    pub close_to_tray: bool,
    pub exclude_from_screen_capture: bool,
    pub notify_on_session_end: bool,
    /// Global shortcut that immediately disables automated input.
    pub emergency_stop_shortcut: String,
    /// Opt-in local activity statistics. Off by default.
    pub record_activity_statistics: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            launch_at_login: false,
            close_to_tray: true,
            exclude_from_screen_capture: false,
            notify_on_session_end: true,
            emergency_stop_shortcut: DEFAULT_EMERGENCY_SHORTCUT.to_string(),
            record_activity_statistics: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub settings: AppSettings,
    /// Built-ins first, then custom profiles; only custom ones are stored.
    #[serde(default, serialize_with = "serialize_custom_profiles")]
    pub profiles: Vec<ActivityProfile>,
    #[serde(default)]
    pub last_session_config: Option<SessionConfig>,
    #[serde(default)]
    pub templates: Vec<SessionTemplate>,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

fn current_schema_version() -> u32 {
    1
}

fn serialize_custom_profiles<S: Serializer>(
    profiles: &[ActivityProfile],
    s: S,
) -> Result<S::Ok, S::Error> {
    s.collect_seq(profiles.iter().filter(|p| !p.built_in))
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: current_schema_version(),
            settings: AppSettings::default(),
            profiles: built_in_profiles(),
            last_session_config: None,
            templates: Vec::new(),
            history: Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read or write configuration: {0}")]
    Io(#[from] io::Error),
    #[error("configuration file is corrupt: {0}")]
    Corrupt(#[from] serde_json::Error),
}

impl AppConfig {
    /// Loads `dir/config.json`; a missing file yields defaults, a corrupt one errors.
    pub fn load(dir: &Path) -> Result<Self, ConfigError> {
        let path = dir.join(CONFIG_FILE_NAME);
        let mut config: Self = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents)?,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(err) => return Err(err.into()),
        };
        config.adopt_current_profiles();
        Ok(config)
    }

    /// Loads the config, moving an unreadable file aside instead of losing it.
    /// Returns a user-facing warning when that happened.
    pub fn load_or_recover(dir: &Path) -> (Self, Option<String>) {
        match Self::load(dir) {
            Ok(config) => (config, None),
            Err(err) => {
                let path = dir.join(CONFIG_FILE_NAME);
                let stamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                let backup = dir.join(format!("config.corrupt-{stamp}.json"));
                let warning = match fs::rename(&path, &backup) {
                    Ok(()) => format!(
                        "Your settings couldn't be read ({err}), so Awake started with defaults. \
                         The old file was saved as {}.",
                        backup.display()
                    ),
                    Err(_) => format!(
                        "Your settings couldn't be read ({err}), so Awake started with defaults."
                    ),
                };
                tracing::error!("{warning}");
                (Self::default(), Some(warning))
            }
        }
    }

    /// Replaces stored built-ins with the current ones and repairs legacy custom profiles.
    fn adopt_current_profiles(&mut self) {
        let built_ins = built_in_profiles();
        self.profiles
            .retain(|p| !p.built_in && built_ins.iter().all(|b| b.id != p.id));
        for profile in &mut self.profiles {
            profile.migrate();
        }
        self.profiles.splice(0..0, built_ins);
    }

    /// Writes `dir/config.json` durably via a synced temp file and rename.
    pub fn save(&self, dir: &Path) -> Result<(), ConfigError> {
        fs::create_dir_all(dir)?;
        let path = dir.join(CONFIG_FILE_NAME);
        let tmp_path = tmp_path_for(&path);

        let contents = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&tmp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp_path, &path)?;
        // Already saved; only log this.
        #[cfg(unix)]
        if let Err(err) = File::open(dir).and_then(|d| d.sync_all()) {
            tracing::warn!("couldn't sync config directory: {err}");
        }
        Ok(())
    }
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    tmp.set_extension("json.tmp");
    tmp
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_yields_defaults() {
        let dir = tempdir().unwrap();
        let config = AppConfig::load(dir.path()).unwrap();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempdir().unwrap();
        let mut config = AppConfig::default();
        config.settings.start_minimized = true;
        config.settings.record_activity_statistics = true;

        config.save(dir.path()).unwrap();
        let loaded = AppConfig::load(dir.path()).unwrap();

        assert_eq!(config, loaded);
    }

    #[test]
    fn corrupt_file_is_reported_not_silently_replaced() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILE_NAME), "{ not valid json").unwrap();

        let result = AppConfig::load(dir.path());
        assert!(matches!(result, Err(ConfigError::Corrupt(_))));
    }

    #[test]
    fn a_corrupt_file_is_moved_aside_with_a_warning() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(CONFIG_FILE_NAME), "{ not valid json").unwrap();

        let (config, warning) = AppConfig::load_or_recover(dir.path());

        assert_eq!(config, AppConfig::default());
        assert!(warning.is_some());
        assert!(!dir.path().join(CONFIG_FILE_NAME).exists());
        let kept: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("config.corrupt-")
            })
            .collect();
        assert_eq!(kept.len(), 1);
        assert_eq!(
            fs::read_to_string(kept[0].path()).unwrap(),
            "{ not valid json"
        );
    }

    #[test]
    fn only_custom_profiles_are_stored() {
        let dir = tempdir().unwrap();
        let mut config = AppConfig::default();
        config
            .profiles
            .push(ActivityProfile::new_custom("custom-1".to_string()));
        config.save(dir.path()).unwrap();

        let raw: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(CONFIG_FILE_NAME)).unwrap())
                .unwrap();
        let stored = raw["profiles"].as_array().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0]["id"], "custom-1");
    }

    #[test]
    fn stale_stored_built_ins_are_replaced_and_customs_migrated() {
        let dir = tempdir().unwrap();
        let mut stale = built_in_profiles().remove(2);
        stale.gestures.per_minute = 1;
        let mut legacy = ActivityProfile::new_custom("custom-1".to_string());
        legacy.keyboard.key = "A".to_string();
        let json = serde_json::json!({ "profiles": [stale, legacy] });
        fs::write(dir.path().join(CONFIG_FILE_NAME), json.to_string()).unwrap();

        let config = AppConfig::load(dir.path()).unwrap();

        assert_eq!(config.profiles[..3], built_in_profiles()[..]);
        assert_eq!(config.profiles.len(), 4);
        assert_eq!(
            config.profiles[3].keyboard.key,
            crate::core::profiles::DEFAULT_KEY
        );
    }

    #[test]
    fn save_does_not_leave_a_temp_file_behind() {
        let dir = tempdir().unwrap();
        AppConfig::default().save(dir.path()).unwrap();
        let tmp = dir.path().join("config.json.tmp");
        assert!(!tmp.exists());
    }

    #[test]
    fn old_config_missing_new_fields_still_loads_with_defaults() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILE_NAME), "{}").unwrap();

        let config = AppConfig::load(dir.path()).unwrap();
        assert_eq!(config.settings, AppSettings::default());
        assert_eq!(config.profiles.len(), built_in_profiles().len());
    }
}
