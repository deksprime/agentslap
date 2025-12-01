//! Integration tests for Phase 0
//!
//! These tests verify that all Phase 0 components work together correctly.

use agent_core::{
    config::{load_config_or_default, AgentConfig},
    error::{AgentError, Result},
    logging::LogConfig,
};

#[test]
fn test_config_loading() {
    // Should load defaults when file doesn't exist
    let config = load_config_or_default("nonexistent.toml");
    assert_eq!(config.agent.name, "agent");
    assert_eq!(config.logging.level, "info");
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = AgentConfig::default();

    // Serialize to JSON
    let json = serde_json::to_string(&config).expect("Failed to serialize");

    // Deserialize back
    let deserialized: AgentConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(config.agent.name, deserialized.agent.name);
    assert_eq!(
        config.agent.max_iterations,
        deserialized.agent.max_iterations
    );
}

#[test]
fn test_error_handling() {
    let result: Result<()> = Err(AgentError::config("test error"));
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("test error"));
    }
}

#[test]
fn test_error_conversion() {
    // Test IO error conversion
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let agent_err = AgentError::from(io_err);
    assert!(matches!(agent_err, AgentError::Io(_)));
}

#[test]
fn test_logging_config() {
    let config = LogConfig {
        level: "debug".to_string(),
        json: true,
    };

    assert_eq!(config.level, "debug");
    assert!(config.json);
}

#[test]
fn test_agent_config_defaults() {
    let config = AgentConfig::default();

    assert_eq!(config.logging.level, "info");
    assert!(!config.logging.json);
    assert_eq!(config.agent.name, "agent");
    assert_eq!(config.agent.max_iterations, 10);
}

#[test]
fn test_custom_agent_config() {
    let json = r#"{
        "logging": {
            "level": "trace",
            "json": false
        },
        "agent": {
            "name": "custom-agent",
            "max_iterations": 50
        }
    }"#;

    let config: AgentConfig = serde_json::from_str(json).expect("Failed to parse JSON");

    assert_eq!(config.logging.level, "trace");
    assert_eq!(config.agent.name, "custom-agent");
    assert_eq!(config.agent.max_iterations, 50);
}
