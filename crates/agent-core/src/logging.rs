//! Logging configuration for the agent runtime
//!
//! This module sets up structured logging using the `tracing` crate,
//! which is async-aware and provides excellent performance.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (e.g., "info", "debug", "trace")
    pub level: String,
    /// Whether to use JSON format (vs. human-readable)
    pub json: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json: false,
        }
    }
}

/// Initialize logging for the application
///
/// This sets up a tracing subscriber with the specified configuration.
/// Should be called once at application startup.
///
/// # Example
///
/// ```
/// use agent_core::logging::{init_logging, LogConfig};
///
/// let config = LogConfig {
///     level: "debug".to_string(),
///     json: false,
/// };
/// init_logging(config);
/// ```
pub fn init_logging(config: LogConfig) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    if config.json {
        // JSON format for production/structured logging
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .init();
    } else {
        // Human-readable format for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().pretty())
            .init();
    }

    tracing::info!("Logging initialized at level: {}", config.level);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.json);
    }

    #[test]
    fn test_custom_config() {
        let config = LogConfig {
            level: "debug".to_string(),
            json: true,
        };
        assert_eq!(config.level, "debug");
        assert!(config.json);
    }
}
