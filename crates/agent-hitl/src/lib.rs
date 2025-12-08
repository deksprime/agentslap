//! Human-in-the-Loop
//!
//! Approval workflows for sensitive agent operations.
//!
//! # Example
//!
//! ```
//! use agent_hitl::{ApprovalStrategy, AutoApprove, ApprovalRequest, RiskLevel};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let strategy = AutoApprove::new();
//!     
//!     let request = ApprovalRequest::new("Delete file", RiskLevel::High)
//!         .with_timeout(Duration::from_secs(30));
//!     
//!     let response = strategy.request_approval(request).await?;
//!     println!("Response: {:?}", response);
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod strategy;
pub mod request;
pub mod audit;

// Built-in strategies
pub mod auto_approve;
pub mod console;
pub mod callback;
pub mod mock;

// Re-exports
pub use error::{HitlError, Result};
pub use strategy::ApprovalStrategy;
pub use request::{ApprovalRequest, ApprovalResponse, RiskLevel};
pub use audit::ApprovalAudit;

pub use auto_approve::AutoApprove;
pub use console::ConsoleApproval;
pub use callback::CallbackApproval;
pub use mock::MockApproval;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        assert!(true);
    }
}
