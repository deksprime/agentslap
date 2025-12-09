//! Tool Calling System
//!
//! This crate provides the tool/function calling infrastructure for agents,
//! enabling them to perform actions and gather information.
//!
//! # Example
//!
//! ```
//! use agent_tools::{Tool, ToolRegistry};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut registry = ToolRegistry::new();
//!     
//!     // Register tools
//!     // registry.register(calculator_tool);
//!     
//!     // Execute tool
//!     // let result = registry.execute("calculator", json!({"a": 5, "b": 3})).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod tool;
pub mod registry;
pub mod schema;

// Built-in tools
pub mod builtin;

// Re-exports
pub use error::{ToolError, Result};
pub use tool::{Tool, ToolResult, ToolRiskLevel};
pub use registry::ToolRegistry;
pub use schema::ToolSchema;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify main types are accessible
        let registry = ToolRegistry::new();
        assert_eq!(registry.count(), 0);
    }
}
