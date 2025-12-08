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
    println!("  ✓ Foundation (error handling, logging, config)");
    println!("  ✓ LLM Providers (OpenAI, Anthropic)");
    println!("  ✓ Conversations (multi-turn, token management)");
    println!("  ✓ Sessions (memory, cache, layered storage)");
    println!("  ✓ Tools (calculator, echo, current_time)");
    println!("  ✓ Agent Runtime (complete agent loop)");
    println!("\n Run examples:");
    println!("  cargo run -p agent-runtime --example basic_agent");
}
