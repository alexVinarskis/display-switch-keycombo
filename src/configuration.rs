//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use crate::input_source::InputSource;
use crate::key_combination::KeyCombination;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum SwitchDirection {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InputSources {
    // Note: Serde alias won't work here, because of https://github.com/serde-rs/serde/issues/1504
    // So cannot alias (ex) "on_usb_connect" to "monitor_input"
    pub input_a: Option<InputSource>,
    pub input_b: Option<InputSource>,
    pub input_c: Option<InputSource>,
    pub input_d: Option<InputSource>,
    pub input_a_execute: Option<String>,
    pub input_b_execute: Option<String>,
    pub input_c_execute: Option<String>,
    pub input_d_execute: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PerMonitorConfiguration {
    monitor_id: String,
    #[serde(flatten)]
    input_sources: InputSources,
}

#[derive(Debug, Deserialize)]
pub struct KeyCombinations {
    pub combo_a: Option<KeyCombination>,
    pub combo_b: Option<KeyCombination>,
    pub combo_c: Option<KeyCombination>,
    pub combo_d: Option<KeyCombination>,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    #[serde(flatten)]
    pub key_combinations: KeyCombinations,
    #[serde(flatten)]
    pub default_input_sources: InputSources,
    monitor1: Option<PerMonitorConfiguration>,
    monitor2: Option<PerMonitorConfiguration>,
    monitor3: Option<PerMonitorConfiguration>,
    monitor4: Option<PerMonitorConfiguration>,
    monitor5: Option<PerMonitorConfiguration>,
    monitor6: Option<PerMonitorConfiguration>,
}

impl fmt::Display for SwitchDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => write!(f, "Switched to input A"),
            Self::B => write!(f, "Switched to input B"),
            Self::C => write!(f, "Switched to input C"),
            Self::D => write!(f, "Switched to input D"),
        }
    }
}

impl PerMonitorConfiguration {
    fn matches(&self, monitor_id: &str) -> bool {
        monitor_id.to_lowercase().contains(&self.monitor_id.to_lowercase())
    }
}

impl InputSources {
    fn merge(&self, default: &Self) -> Self {
        Self {
            // Global configuration for execution is not merged! Otherwise, for two
            // monitors, we'll be executing the same command twice. Global config is treated
            // separately during switching.
            input_a: self.input_a.or(default.input_a),
            input_b: self.input_b.or(default.input_b),
            input_c: self.input_c.or(default.input_c),
            input_d: self.input_d.or(default.input_d),
            input_a_execute: self.input_a_execute.clone(),
            input_b_execute: self.input_b_execute.clone(),
            input_c_execute: self.input_c_execute.clone(),
            input_d_execute: self.input_d_execute.clone(),
        }
    }

    pub fn source(&self, direction: SwitchDirection) -> Option<InputSource> {
        match direction {
            SwitchDirection::A => self.input_a,
            SwitchDirection::B => self.input_b,
            SwitchDirection::C => self.input_c,
            SwitchDirection::D => self.input_d,
        }
    }

    pub fn execute_command(&self, direction: SwitchDirection) -> Option<&str> {
        match direction {
            SwitchDirection::A => self.input_a_execute.as_ref().map(|x| &**x),
            SwitchDirection::B => self.input_b_execute.as_ref().map(|x| &**x),
            SwitchDirection::C => self.input_c_execute.as_ref().map(|x| &**x),
            SwitchDirection::D => self.input_d_execute.as_ref().map(|x| &**x),
        }
    }
}

impl Configuration {
    pub fn load() -> Result<Self> {
        let config_file_name = Self::config_file_name()?;
        let builder = config::Config::builder()
            .add_source(config::File::from(config_file_name.clone()))
            .add_source(config::Environment::with_prefix("DISPLAY_SWITCH_KEYCOMBO"));

        let config = builder.build()?.try_deserialize()?;
        info!("Configuration loaded ({:?}): {:?}", config_file_name, config);
        Ok(config)
    }

    pub fn config_file_name() -> Result<std::path::PathBuf> {
        let config_dir = if cfg!(target_os = "macos") {
            dirs::preference_dir().ok_or_else(|| anyhow!("Config directory not found"))?
        } else {
            dirs::config_dir()
                .ok_or_else(|| anyhow!("Config directory not found"))?
                .join("display-switch-keycombo")
        };
        std::fs::create_dir_all(&config_dir)
            .with_context(|| format!("failed to create directory: {:?}", config_dir))?;
        Ok(config_dir.join("display-switch-keycombo.ini"))
    }

    pub fn log_file_name() -> Result<std::path::PathBuf> {
        let log_dir = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or_else(|| anyhow!("Home directory not found"))?
                .join("Library")
                .join("Logs")
                .join("display-switch-keycombo")
        } else {
            dirs::data_local_dir()
                .ok_or_else(|| anyhow!("Data-local directory not found"))?
                .join("display-switch-keycombo")
        };
        std::fs::create_dir_all(&log_dir).with_context(|| format!("failed to create directory: {:?}", log_dir))?;
        Ok(log_dir.join("display-switch-keycombo.log"))
    }

    pub fn configuration_for_monitor(&self, monitor_id: &str) -> InputSources {
        // Find a matching per-monitor config, if there is any
        let per_monitor_config = [
            &self.monitor1,
            &self.monitor2,
            &self.monitor3,
            &self.monitor4,
            &self.monitor5,
            &self.monitor6,
        ]
        .iter()
        .find_map(|config| {
            config
                .as_ref()
                .and_then(|config| if config.matches(monitor_id) { Some(config) } else { None })
        });
        // Merge global config as needed
        per_monitor_config.map_or(
            InputSources {
                input_a: self.default_input_sources.input_a,
                input_b: self.default_input_sources.input_b,
                input_c: self.default_input_sources.input_c,
                input_d: self.default_input_sources.input_d,
                input_a_execute: None,
                input_b_execute: None,
                input_c_execute: None,
                input_d_execute: None,
            },
            |config| config.input_sources.merge(&self.default_input_sources),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ConfigError;
    use config::FileFormat::Ini;
    use rdev::Key;
    use KeyCombination;

    #[test]
    fn test_log_file_name() {
        let file_name = Configuration::log_file_name();
        assert!(file_name.is_ok());
        assert!(file_name.unwrap().ends_with("display-switch-keycombo.log"))
    }

    fn load_test_config(config_str: &str) -> Result<Configuration, ConfigError> {
        config::Config::builder()
            .add_source(config::File::from_str(config_str, Ini))
            .build()?
            .try_deserialize()
    }

    #[test]
    fn test_usb_device_deserialization() {
        let config = load_test_config(
            r#"
            combo_a = ShiftLeft ControlLeft Comma
            on_usb_connect = "DisplayPort2"
        "#,
        )
        .unwrap();
        assert_eq!(config.key_combinations.combo_a.unwrap(), KeyCombination::new(vec![Key::ShiftLeft, Key::ControlLeft, Key::Comma]));
    }

    #[test]
    fn test_symbolic_input_deserialization() {
        let config = load_test_config(
            r#"
            combo_a = ShiftLeft ControlLeft Comma
            input_a = "DisplayPort2"
            input_d = DisplayPort1
        "#,
        )
        .unwrap();
        assert_eq!(config.default_input_sources.input_a.unwrap().value(), 0x10);
        assert_eq!(config.default_input_sources.input_d.unwrap().value(), 0x0f);
    }

    #[test]
    fn test_decimal_input_deserialization() {
        let config = load_test_config(
            r#"
            combo_a = ShiftLeft ControlLeft Comma
            input_a = 22
            input_c = 33
        "#,
        )
        .unwrap();
        assert_eq!(config.default_input_sources.input_a.unwrap().value(), 22);
        assert_eq!(config.default_input_sources.input_c.unwrap().value(), 33);
    }

    #[test]
    fn test_hexadecimal_input_deserialization() {
        let config = load_test_config(
            r#"
            combo_a = ShiftLeft ControlLeft Comma
            input_a = "0x10"
            input_b = "0x20"
        "#,
        )
        .unwrap();
        assert_eq!(config.default_input_sources.input_a.unwrap().value(), 0x10);
        assert_eq!(config.default_input_sources.input_b.unwrap().value(), 0x20);
    }

    #[test]
    fn test_per_monitor_config() {
        let config = load_test_config(
            r#"
            combo_a = ShiftLeft ControlLeft Comma
            input_a = "0x10"
            input_d = "0x20"
            input_a_execute = "foo"

            [monitor1]
            monitor_id = 123
            input_a = 0x11
            input_d_execute = "bar"

            [monitor2]
            monitor_id = 45
            input_a = 0x12
            input_d = 0x13
        "#,
        )
        .unwrap();

        // When no specific monitor matches, use the global defaults
        assert_eq!(
            config.configuration_for_monitor("333").input_a.unwrap().value(),
            0x10
        );
        // Matches monitor #1, and it should use its "input_a" and global "input_d"
        assert_eq!(
            config.configuration_for_monitor("1234").input_a.unwrap().value(),
            0x11
        );
        assert_eq!(
            config
                .configuration_for_monitor("1234")
                .input_d
                .unwrap()
                .value(),
            0x20
        );
        // Matches monitor #2, and it should use its "input_a" and "input_d" values
        assert_eq!(
            config.configuration_for_monitor("2345").input_a.unwrap().value(),
            0x12
        );
        assert_eq!(
            config
                .configuration_for_monitor("2345")
                .input_d
                .unwrap()
                .value(),
            0x13
        );
        // Optional "run command" on "input_a_execute" / "input_d_execute"
        assert_eq!(config.configuration_for_monitor("123").input_a_execute, None);
        assert_eq!(
            config.configuration_for_monitor("123").input_d_execute,
            Some("bar".into())
        );
    }
}
