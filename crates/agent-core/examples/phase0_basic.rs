//! Phase 0 Example: Basic Configuration and Logging
//!
//! This example demonstrates:
//! - Loading configuration (with defaults)
//! - Initializing logging
//! - Error handling
//! - Basic application structure
//!
//! Run with: cargo run --example phase0_basic

use agent_core::{
    config::load_config_or_default,
    error::Result,
    logging::{init_logging, LogConfig},
};
use tracing::{debug, error, info, warn};

fn main() -> Result<()> {
    // Initialize logging first (using default config)
    let log_config = LogConfig {
        level: "debug".to_string(),
        json: false,
    };
    init_logging(log_config);

    info!("🚀 Starting Phase 0 example");

    // Load configuration (will use defaults if file doesn't exist)
    let config = load_config_or_default("config.toml");
    debug!("Configuration loaded: {:?}", config);

    // Demonstrate different log levels
    info!("Agent name: {}", config.agent.name);
    debug!("Max iterations: {}", config.agent.max_iterations);
    warn!("This is a warning message");

    // Demonstrate error handling
    match risky_operation() {
        Ok(value) => info!("Operation succeeded with value: {}", value),
        Err(e) => error!("Operation failed: {}", e),
    }

    // Demonstrate successful error handling
    let result = safe_operation()?;
    info!("Safe operation result: {}", result);

    info!("✅ Phase 0 example completed successfully");

    Ok(())
}

/// A function that might fail
fn risky_operation() -> Result<i32> {
    // Simulate an error
    Err(agent_core::error::AgentError::other(
        "Simulated error for demonstration",
    ))
}

/// A function that succeeds
fn safe_operation() -> Result<String> {
    Ok("Success!".to_string())
}
