//! Agent Communication Layer
//!
//! Enables multi-agent collaboration with location transparency.
//!
//! # Example
//!
//! ```no_run
//! use agent_comms::{AgentAddress, AgentMessage, InProcessTransport, MessageTransport};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let transport = InProcessTransport::new();
//!     
//!     let addr = AgentAddress::local("agent-1");
//!     let message = AgentMessage::new("agent-1", "agent-2", serde_json::json!({"text": "Hello!"}));
//!     
//!     transport.send(&addr, message).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod address;
pub mod message;
pub mod transport;
pub mod registry;
pub mod context;
pub mod coordinator;

// In-process transport
pub mod in_process;

// Re-exports
pub use error::{CommsError, Result};
pub use address::{AgentAddress, AgentLocation};
pub use message::{AgentMessage, MessageType};
pub use transport::MessageTransport;
pub use registry::AgentRegistry;
pub use context::AgentContext;
pub use coordinator::AgentCoordinator;

pub use in_process::InProcessTransport;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        assert!(true);
    }
}
