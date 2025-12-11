//! Agent coordination and orchestration
//!
//! This crate handles multi-agent spawning, lifecycle management,
//! and coordination with the AgentCoordinator.

pub mod factory;
pub mod orchestrator;
pub mod error;

// Re-exports
pub use error::{CoordinationError, Result};
pub use factory::{AgentFactory, RoleConfig};
pub use orchestrator::{spawn_agent_with_loop, AgentRunHandle};

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        assert!(true);
    }
}
