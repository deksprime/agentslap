//! Agent Core
//!
//! This crate provides the core functionality for the agent runtime,
//! including error handling, configuration, and logging setup.

pub mod config;
pub mod error;
pub mod logging;

// Re-export commonly used types
pub use config::{load_config, AgentConfig};
pub use error::{AgentError, Result};
pub use logging::init_logging;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Basic smoke test - verify module exports are accessible
        let config = AgentConfig::default();
        assert_eq!(config.agent.name, "agent");
    }
}
