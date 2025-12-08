//! Content filtering guardrail

use async_trait::async_trait;
use regex::Regex;

use crate::{
    guardrail::{Guardrail, InputContext, OutputContext, ToolCallContext},
    violation::{ViolationAction, ViolationSeverity},
    Result, Violation,
};

/// Content filter guardrail
///
/// Blocks messages containing specified patterns or phrases.
pub struct ContentFilter {
    /// Blocked patterns (regex)
    patterns: Vec<Regex>,
    /// Blocked exact phrases
    blocked_phrases: Vec<String>,
    /// Case sensitive matching
    case_sensitive: bool,
}

impl ContentFilter {
    /// Create a new content filter with blocked words/phrases
    pub fn new(blocked: Vec<String>) -> Self {
        Self {
            patterns: Vec::new(),
            blocked_phrases: blocked,
            case_sensitive: false,
        }
    }

    /// Add a regex pattern to block
    pub fn with_pattern(mut self, pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| crate::GuardrailError::error(format!("Invalid regex: {}", e)))?;
        self.patterns.push(regex);
        Ok(self)
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, enabled: bool) -> Self {
        self.case_sensitive = enabled;
        self
    }

    /// Check text against filters
    fn check_text(&self, text: &str) -> Option<String> {
        let text_to_check = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        // Check blocked phrases
        for phrase in &self.blocked_phrases {
            let phrase_to_check = if self.case_sensitive {
                phrase.clone()
            } else {
                phrase.to_lowercase()
            };

            if text_to_check.contains(&phrase_to_check) {
                return Some(format!("Contains blocked phrase: {}", phrase));
            }
        }

        // Check regex patterns
        for pattern in &self.patterns {
            if pattern.is_match(text) {
                return Some(format!("Matches blocked pattern: {}", pattern.as_str()));
            }
        }

        None
    }
}

#[async_trait]
impl Guardrail for ContentFilter {
    fn name(&self) -> &str {
        "content_filter"
    }

    async fn check_input(&self, context: &InputContext) -> Result<Option<Violation>> {
        if let Some(reason) = self.check_text(&context.message) {
            return Ok(Some(
                Violation::new(
                    "content_filter",
                    ViolationSeverity::High,
                    &reason,
                    ViolationAction::Block,
                )
            ));
        }
        Ok(None)
    }

    async fn check_output(&self, context: &OutputContext) -> Result<Option<Violation>> {
        if let Some(reason) = self.check_text(&context.response) {
            return Ok(Some(
                Violation::new(
                    "content_filter",
                    ViolationSeverity::High,
                    &reason,
                    ViolationAction::Block,
                )
            ));
        }
        Ok(None)
    }

    async fn check_tool_call(&self, _context: &ToolCallContext) -> Result<Option<Violation>> {
        // Content filter doesn't check tool calls
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blocked_phrase() {
        let filter = ContentFilter::new(vec!["badword".to_string()]);

        let context = InputContext {
            message: "This contains badword in it".to_string(),
            user_id: None,
            session_id: None,
        };

        let violation = filter.check_input(&context).await.unwrap();
        assert!(violation.is_some());
        assert!(violation.unwrap().should_block());
    }

    #[tokio::test]
    async fn test_case_insensitive() {
        let filter = ContentFilter::new(vec!["BLOCKED".to_string()]);

        let context = InputContext {
            message: "This has blocked in lowercase".to_string(),
            user_id: None,
            session_id: None,
        };

        let violation = filter.check_input(&context).await.unwrap();
        assert!(violation.is_some());
    }

    #[tokio::test]
    async fn test_regex_pattern() {
        let filter = ContentFilter::new(vec![])
            .with_pattern(r"\d{3}-\d{2}-\d{4}")  // SSN pattern
            .unwrap();

        let context = InputContext {
            message: "My SSN is 123-45-6789".to_string(),
            user_id: None,
            session_id: None,
        };

        let violation = filter.check_input(&context).await.unwrap();
        assert!(violation.is_some());
    }

    #[tokio::test]
    async fn test_allowed_content() {
        let filter = ContentFilter::new(vec!["blocked".to_string()]);

        let context = InputContext {
            message: "This is fine".to_string(),
            user_id: None,
            session_id: None,
        };

        let violation = filter.check_input(&context).await.unwrap();
        assert!(violation.is_none());
    }

    #[tokio::test]
    async fn test_output_filtering() {
        let filter = ContentFilter::new(vec!["sensitive".to_string()]);

        let context = OutputContext {
            response: "This contains sensitive information".to_string(),
            model: "gpt-4".to_string(),
            tokens_used: None,
        };

        let violation = filter.check_output(&context).await.unwrap();
        assert!(violation.is_some());
    }
}

