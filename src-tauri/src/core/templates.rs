//! Named, reusable session presets: a full `SessionConfig` bundled with a name.

use serde::{Deserialize, Serialize};

use super::session::SessionConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTemplate {
    pub id: String,
    pub name: String,
    pub config: SessionConfig,
}

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum TemplateError {
    #[error("template name must not be empty")]
    EmptyName,
}

impl SessionTemplate {
    pub fn validate(&self) -> Result<(), TemplateError> {
        if self.name.trim().is_empty() {
            return Err(TemplateError::EmptyName);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn sample_config() -> SessionConfig {
        SessionConfig {
            duration: Duration::from_secs(3600),
            keep_system_awake: true,
            keep_display_awake: false,
            inactivity_threshold: Duration::from_secs(300),
            activity_profile_id: None,
        }
    }

    #[test]
    fn rejects_empty_name() {
        let template = SessionTemplate {
            id: "t1".into(),
            name: "  ".into(),
            config: sample_config(),
        };
        assert_eq!(template.validate(), Err(TemplateError::EmptyName));
    }

    #[test]
    fn accepts_a_named_template() {
        let template = SessionTemplate {
            id: "t1".into(),
            name: "Overnight".into(),
            config: sample_config(),
        };
        assert!(template.validate().is_ok());
    }

    #[test]
    fn round_trips_through_json() {
        let template = SessionTemplate {
            id: "t1".into(),
            name: "Overnight".into(),
            config: sample_config(),
        };
        let json = serde_json::to_string(&template).unwrap();
        let restored: SessionTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(template, restored);
    }
}
