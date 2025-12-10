//! Guardrails System
//!
//! Safety and control mechanisms for agent execution.
//!
//! # Example
//!
//! ```
//! use agent_guardrails::{GuardrailChain, ContentFilter, RateLimiter};
//!
//! let chain = GuardrailChain::new()
//!     .with_guardrail(ContentFilter::new(vec!["blocked".to_string()]))
//!     .with_guardrail(RateLimiter::new(100, std::time::Duration::from_secs(60)));
//! ```

pub mod error;
pub mod guardrail;
pub mod chain;
pub mod violation;
pub mod config;

// Built-in guardrails
pub mod content_filter;
pub mod rate_limiter;
pub mod cost_limiter;
pub mod tool_allowlist;

// Re-exports
pub use error::{GuardrailError, Result};
pub use guardrail::Guardrail;
pub use chain::GuardrailChain;
pub use violation::{Violation, ViolationSeverity, ViolationAction};
pub use config::GuardrailConfig;

pub use content_filter::ContentFilter;
pub use rate_limiter::RateLimiter;
pub use cost_limiter::CostLimiter;
pub use tool_allowlist::ToolAllowlist;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        assert!(true);
    }
}
