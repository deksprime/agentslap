//! Agent Runtime CLI
//!
//! Command-line interface for the agent runtime.

use agent_core::{
    config::load_config_or_default,
    logging::{init_logging, LogConfig},
};

fn main() {
    // Initialize logging
    let log_config = LogConfig {
        level: "info".to_string(),
        json: false,
    };
    init_logging(log_config);

    // Load configuration
    let config = load_config_or_default("config.toml");

    println!("🤖 Agent Runtime v{}", env!("CARGO_PKG_VERSION"));
    println!("Agent: {}", config.agent.name);
    println!("\nPhase 0 complete! Ready for Phase 1.");
    println!("\nNext steps:");
    println!("  - Phase 1: LLM Provider Abstraction");
    println!("  - See PHASE0_CHECKLIST.md for details");
}
