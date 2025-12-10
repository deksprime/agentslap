//! Approval request and response types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Risk level for an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk - read operations, safe calculations
    Low,
    /// Medium risk - write operations, API calls
    Medium,
    /// High risk - delete operations, external access
    High,
    /// Critical risk - system-level operations
    Critical,
}

/// Approval request for a human decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier for tracking
    pub id: String,
    
    /// Human-readable description of the action
    pub action: String,
    
    /// Risk level of the action
    pub risk_level: RiskLevel,
    
    /// Additional context (tool params, cost estimate, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
    
    /// Timeout for approval
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    
    /// When the request was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new<S: Into<String>>(action: S, risk_level: RiskLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            action: action.into(),
            risk_level,
            context: None,
            timeout: Duration::from_secs(30),
            created_at: chrono::Utc::now(),
        }
    }

    /// Set context
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Response to an approval request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ApprovalResponse {
    /// Approved - proceed with action
    Approved,
    
    /// Denied - do not proceed
    Denied {
        /// Reason for denial
        reason: Option<String>,
    },
    
    /// Modified - proceed with modified parameters
    Modified {
        /// Modified context/parameters
        modified_context: serde_json::Value,
    },
    
    /// Timeout - no response within timeout period
    Timeout,
}

impl ApprovalResponse {
    /// Check if the request was approved
    pub fn is_approved(&self) -> bool {
        matches!(self, ApprovalResponse::Approved | ApprovalResponse::Modified { .. })
    }

    /// Check if the request was denied
    pub fn is_denied(&self) -> bool {
        matches!(self, ApprovalResponse::Denied { .. })
    }

    /// Check if the request timed out
    pub fn is_timeout(&self) -> bool {
        matches!(self, ApprovalResponse::Timeout)
    }
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_request_creation() {
        let req = ApprovalRequest::new("Test action", RiskLevel::Medium);
        assert_eq!(req.risk_level, RiskLevel::Medium);
        assert!(!req.id.is_empty());
    }

    #[test]
    fn test_approval_response_checks() {
        assert!(ApprovalResponse::Approved.is_approved());
        assert!(!ApprovalResponse::Approved.is_denied());
        
        assert!(ApprovalResponse::Denied { reason: None }.is_denied());
        assert!(!ApprovalResponse::Denied { reason: None }.is_approved());
        
        assert!(ApprovalResponse::Timeout.is_timeout());
    }

    #[test]
    fn test_request_serialization() {
        let req = ApprovalRequest::new("Test", RiskLevel::High)
            .with_context(serde_json::json!({"key": "value"}));

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ApprovalRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.risk_level, RiskLevel::High);
        assert!(deserialized.context.is_some());
    }

    #[test]
    fn test_response_serialization() {
        let response = ApprovalResponse::Approved;
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ApprovalResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, ApprovalResponse::Approved);
    }
}

