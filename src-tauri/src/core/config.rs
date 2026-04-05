//! Local configuration persistence: one JSON file in the OS app-config directory.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::profiles::{built_in_profiles, ActivityProfile};

pub const CONFIG_FILE_NAME: &str = "config.json";

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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_minimized: false,
            launch_at_login: false,
            close_to_tray: true,
            exclude_from_screen_capture: false,
            notify_on_session_end: true,
            emergency_stop_shortcut: "CommandOrControl+Shift+Escape".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub settings: AppSettings,
    #[serde(default = "built_in_profiles")]
    pub profiles: Vec<ActivityProfile>,
}

fn current_schema_version() -> u32 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: current_schema_version(),
            settings: AppSettings::default(),
            profiles: built_in_profiles(),
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
        match fs::read_to_string(&path) {
            Ok(contents) => Ok(serde_json::from_str(&contents)?),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn save(&self, dir: &Path) -> Result<(), ConfigError> {
        fs::create_dir_all(dir)?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(dir.join(CONFIG_FILE_NAME), contents)?;
        Ok(())
    }
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
    fn old_config_missing_new_fields_still_loads_with_defaults() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path()).unwrap();
        fs::write(dir.path().join(CONFIG_FILE_NAME), "{}").unwrap();

        let config = AppConfig::load(dir.path()).unwrap();
        assert_eq!(config.settings, AppSettings::default());
        assert_eq!(config.profiles.len(), built_in_profiles().len());
    }
}
