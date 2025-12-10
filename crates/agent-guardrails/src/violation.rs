//! Guardrail violations

use serde::{Deserialize, Serialize};

/// Severity of a guardrail violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    /// Low severity - log only
    Low,
    /// Medium severity - warn but allow
    Medium,
    /// High severity - block the action
    High,
    /// Critical severity - block and alert
    Critical,
}

/// Action to take on violation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViolationAction {
    /// Just log the violation
    Log,
    /// Warn but allow to proceed
    Warn,
    /// Block the action
    Block,
}

/// A guardrail violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Name of the guardrail that triggered
    pub guardrail: String,
    
    /// Severity of the violation
    pub severity: ViolationSeverity,
    
    /// Human-readable description
    pub message: String,
    
    /// Recommended action
    pub action: ViolationAction,
    
    /// When the violation occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Additional context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl Violation {
    /// Create a new violation
    pub fn new<S: Into<String>>(
        guardrail: S,
        severity: ViolationSeverity,
        message: S,
        action: ViolationAction,
    ) -> Self {
        Self {
            guardrail: guardrail.into(),
            severity,
            message: message.into(),
            action,
            timestamp: chrono::Utc::now(),
            context: None,
        }
    }

    /// Add context to the violation
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Check if this violation should block execution
    pub fn should_block(&self) -> bool {
        matches!(self.action, ViolationAction::Block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let violation = Violation::new(
            "test_guard",
            ViolationSeverity::High,
            "Test violation",
            ViolationAction::Block,
        );

        assert_eq!(violation.guardrail, "test_guard");
        assert_eq!(violation.severity, ViolationSeverity::High);
        assert!(violation.should_block());
    }

    #[test]
    fn test_violation_with_context() {
        let violation = Violation::new(
            "test",
            ViolationSeverity::Low,
            "Test",
            ViolationAction::Log,
        )
        .with_context(serde_json::json!({"key": "value"}));

        assert!(violation.context.is_some());
        assert!(!violation.should_block());
    }

    #[test]
    fn test_violation_serialization() {
        let violation = Violation::new(
            "test",
            ViolationSeverity::Medium,
            "Test message",
            ViolationAction::Warn,
        );

        let json = serde_json::to_string(&violation).unwrap();
        let deserialized: Violation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.guardrail, "test");
        assert_eq!(deserialized.severity, ViolationSeverity::Medium);
    }
}

