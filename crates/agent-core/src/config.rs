//! Configuration management for the agent runtime
//!
//! This module provides configuration loading from multiple sources:
//! - Default values
//! - Configuration files (TOML, JSON, YAML)
//! - Environment variables
//! - Command-line arguments (future)

use crate::error::{AgentError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration for the agent runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Logging configuration
    pub logging: LoggingConfig,

    /// Agent-specific settings
    pub agent: AgentSettings,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Use JSON format
    #[serde(default)]
    pub json: bool,
}

/// Agent settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    /// Agent name/identifier
    #[serde(default = "default_agent_name")]
    pub name: String,

    /// Maximum iterations for agent loop
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
}

// Default value functions
fn default_log_level() -> String {
    "info".to_string()
}

fn default_agent_name() -> String {
    "agent".to_string()
}

fn default_max_iterations() -> usize {
    10
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig {
                level: default_log_level(),
                json: false,
            },
            agent: AgentSettings {
                name: default_agent_name(),
                max_iterations: default_max_iterations(),
            },
        }
    }
}

/// Load configuration from a file
///
/// Supports TOML, JSON, and YAML formats based on file extension.
///
/// # Example
///
/// ```no_run
/// use agent_core::config::load_config;
///
/// let config = load_config("config.toml").unwrap();
/// println!("Agent name: {}", config.agent.name);
/// ```
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AgentConfig> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(AgentError::config(format!(
            "Config file not found: {}",
            path.display()
        )));
    }

    let settings = config::Config::builder()
        .add_source(config::File::from(path))
        .add_source(config::Environment::with_prefix("AGENT").separator("__"))
        .build()?;

    let config: AgentConfig = settings.try_deserialize()?;

    tracing::info!("Configuration loaded from {}", path.display());

    Ok(config)
}

/// Load configuration with defaults if file doesn't exist
///
/// This is useful for optional configuration files.
pub fn load_config_or_default<P: AsRef<Path>>(path: P) -> AgentConfig {
    match load_config(path) {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to load config, using defaults: {}", e);
            AgentConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.agent.name, "agent");
        assert_eq!(config.agent.max_iterations, 10);
    }

    #[test]
    fn test_config_serialization() {
        let config = AgentConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.agent.name, deserialized.agent.name);
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "logging": {
                "level": "debug",
                "json": true
            },
            "agent": {
                "name": "test-agent",
                "max_iterations": 20
            }
        }"#;

        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.agent.name, "test-agent");
        assert_eq!(config.agent.max_iterations, 20);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_config("nonexistent.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_or_default() {
        let config = load_config_or_default("nonexistent.toml");
        assert_eq!(config.agent.name, "agent");
    }
}
