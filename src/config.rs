/// Configuration system for fasterthefuck.
///
/// Supports customization via TOML config files:
/// - Enable/disable specific rules
/// - Override rule priorities
/// - Global settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Global configuration for fasterthefuck
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,

    /// Per-rule settings
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

/// Global configuration options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Enable interactive selection when multiple corrections available
    #[serde(default = "default_true")]
    pub interactive: bool,

    /// Show debug information
    #[serde(default)]
    pub debug: bool,
}

/// Configuration for a specific rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Enable or disable the rule
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Override the rule's priority (lower = higher priority)
    pub priority: Option<i32>,
}

fn default_true() -> bool {
    true
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            interactive: true,
            debug: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            rules: HashMap::new(),
        }
    }
}

impl Config {
    /// Creates a new empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads config from file. Returns empty config if file doesn't exist.
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Loads config from standard location: ~/.config/fasterthefuck/config.toml
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::default_config_path()?;
        Self::load_from_file(&config_path)
    }

    /// Gets the default config file path
    pub fn default_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("fasterthefuck");

        Ok(config_dir.join("config.toml"))
    }

    /// Saves config to file
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Checks if a rule is enabled (default: true if not specified)
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        self.rules
            .get(rule_name)
            .map(|config| config.enabled)
            .unwrap_or(true)
    }

    /// Gets rule priority override (returns None if not overridden)
    pub fn get_rule_priority(&self, rule_name: &str) -> Option<i32> {
        self.rules.get(rule_name).and_then(|config| config.priority)
    }

    /// Gets example config with documentation
    pub fn example() -> String {
        r#"# FastertTheFuck Configuration

[global]
# Enable interactive selection when multiple corrections are available
interactive = true

# Show debug information
debug = false

# Override rules by name
[rules.git_branch_delete]
# Disable this rule
enabled = false

[rules.git_push_set_upstream]
# Override the priority (lower = higher priority)
priority = 300

[rules.mkdir_p]
enabled = true
priority = 150
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.global.interactive);
        assert!(!config.global.debug);
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_rule_config_default() {
        let config = Config::default();
        assert!(config.is_rule_enabled("any_rule"));
        assert_eq!(config.get_rule_priority("any_rule"), None);
    }

    #[test]
    fn test_rule_disabled() {
        let mut config = Config::default();
        config.rules.insert(
            "git_branch_delete".to_string(),
            RuleConfig {
                enabled: false,
                priority: None,
            },
        );

        assert!(!config.is_rule_enabled("git_branch_delete"));
        assert!(config.is_rule_enabled("other_rule"));
    }

    #[test]
    fn test_rule_priority_override() {
        let mut config = Config::default();
        config.rules.insert(
            "mkdir_p".to_string(),
            RuleConfig {
                enabled: true,
                priority: Some(150),
            },
        );

        assert_eq!(config.get_rule_priority("mkdir_p"), Some(150));
        assert_eq!(config.get_rule_priority("other_rule"), None);
    }

    #[test]
    fn test_config_toml_parsing() {
        let toml_str = r#"
[global]
interactive = true
debug = false

[rules.git_branch_delete]
enabled = false

[rules.mkdir_p]
priority = 200
"#;

        let config: Config = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert!(config.global.interactive);
        assert!(!config.is_rule_enabled("git_branch_delete"));
        assert_eq!(config.get_rule_priority("mkdir_p"), Some(200));
    }

    #[test]
    fn test_config_example_valid() {
        let example = Config::example();
        let config: Result<Config, _> = toml::from_str(&example);
        assert!(config.is_ok(), "Example config should be valid TOML");
    }

    #[test]
    fn test_config_nonexistent_file() {
        let config = Config::load_from_file(Path::new("/nonexistent/path/config.toml"));
        assert!(config.is_ok(), "Should return default config for nonexistent file");
        assert_eq!(config.unwrap(), Config::default());
    }
}
