//! Console-based approval strategy

use async_trait::async_trait;
use std::io::{self, Write};

use crate::{ApprovalRequest, ApprovalResponse, ApprovalStrategy, Result};

/// Console approval strategy
///
/// Prompts for approval via stdin/stdout.
/// Use for CLI applications and local development.
pub struct ConsoleApproval {
    /// Auto-approve low risk actions
    auto_approve_low_risk: bool,
}

impl ConsoleApproval {
    /// Create a new console approval strategy
    pub fn new() -> Self {
        Self {
            auto_approve_low_risk: false,
        }
    }

    /// Auto-approve low risk actions without prompting
    pub fn auto_approve_low_risk(mut self, enabled: bool) -> Self {
        self.auto_approve_low_risk = enabled;
        self
    }
}

impl Default for ConsoleApproval {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApprovalStrategy for ConsoleApproval {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse> {
        // Auto-approve low risk if enabled
        if self.auto_approve_low_risk && matches!(request.risk_level, crate::RiskLevel::Low) {
            tracing::debug!("Auto-approved low risk: {}", request.action);
            return Ok(ApprovalResponse::Approved);
        }

        // Print request
        println!("\n🤚 HUMAN APPROVAL REQUIRED");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Action: {}", request.action);
        println!("Risk Level: {:?}", request.risk_level);
        
        if let Some(context) = &request.context {
            println!("Context: {}", serde_json::to_string_pretty(context).unwrap_or_default());
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
        print!("\nApprove this action? [y/N]: ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let response = match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                tracing::info!("Human approved: {}", request.action);
                ApprovalResponse::Approved
            }
            _ => {
                tracing::info!("Human denied: {}", request.action);
                ApprovalResponse::Denied {
                    reason: Some("User denied via console".to_string()),
                }
            }
        };

        Ok(response)
    }

    fn name(&self) -> &str {
        "console"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_creation() {
        let strategy = ConsoleApproval::new();
        assert_eq!(strategy.name(), "console");
    }

    #[test]
    fn test_auto_approve_low_risk_setting() {
        let strategy = ConsoleApproval::new().auto_approve_low_risk(true);
        assert!(strategy.auto_approve_low_risk);
    }

    // Note: Can't easily test actual stdin/stdout interaction in unit tests
    // This would be tested in integration/E2E tests
}

