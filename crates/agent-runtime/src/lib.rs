//! Agent Runtime
//!
//! This crate ties together all components to create a functioning agent.
//!
//! # Example
//!
//! ```no_run
//! use agent_runtime::Agent;
//! use agent_llm::{OpenAIProvider, Conversation};
//! use agent_tools::{ToolRegistry, builtin::*};
//! use agent_session::InMemoryStore;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create components
//!     let provider = OpenAIProvider::new("api-key", "gpt-4o")?;
//!     let mut tools = ToolRegistry::new();
//!     tools.register(CalculatorTool)?;
//!     let store = InMemoryStore::new();
//!     
//!     // Build agent
//!     let agent = Agent::builder()
//!         .provider(provider)
//!         .tools(tools)
//!         .session_store(store)
//!         .build()?;
//!     
//!     // Run agent
//!     let response = agent.run("What is 15 + 27?").await?;
//!     println!("{}", response);
//!     
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod error;
pub mod parser;

// Re-exports
pub use agent::{Agent, AgentBuilder, AgentConfig};
pub use error::{AgentRuntimeError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify exports are accessible
        let _ = std::mem::size_of::<Agent>();
    }
}

