//! Configuration for guardrails

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
    ContentFilter, CostLimiter, GuardrailChain, RateLimiter, Result, ToolAllowlist,
};

/// Configuration for the guardrail system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    /// Enable guardrails globally
    #[serde(default)]
    pub enabled: bool,

    /// Content filter configuration
    #[serde(default)]
    pub content_filter: ContentFilterConfig,

    /// Rate limiter configuration
    #[serde(default)]
    pub rate_limiter: RateLimiterConfig,

    /// Cost limiter configuration
    #[serde(default)]
    pub cost_limiter: CostLimiterConfig,

    /// Tool allowlist configuration
    #[serde(default)]
    pub tool_allowlist: ToolAllowlistConfig,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            content_filter: ContentFilterConfig::default(),
            rate_limiter: RateLimiterConfig::default(),
            cost_limiter: CostLimiterConfig::default(),
            tool_allowlist: ToolAllowlistConfig::default(),
        }
    }
}

/// Content filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilterConfig {
    /// Enable content filter
    #[serde(default)]
    pub enabled: bool,

    /// Blocked phrases
    #[serde(default)]
    pub blocked_phrases: Vec<String>,

    /// Blocked regex patterns
    #[serde(default)]
    pub blocked_patterns: Vec<String>,

    /// Case sensitive matching
    #[serde(default)]
    pub case_sensitive: bool,
}

impl Default for ContentFilterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            blocked_phrases: Vec::new(),
            blocked_patterns: Vec::new(),
            case_sensitive: false,
        }
    }
}

/// Rate limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Enable rate limiter
    #[serde(default)]
    pub enabled: bool,

    /// Maximum requests allowed
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,

    /// Time window in seconds
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u64,
}

fn default_max_requests() -> usize {
    100
}

fn default_window_seconds() -> u64 {
    60
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_requests: default_max_requests(),
            window_seconds: default_window_seconds(),
        }
    }
}

/// Cost limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLimiterConfig {
    /// Enable cost limiter
    #[serde(default)]
    pub enabled: bool,

    /// Maximum tokens per session
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,

    /// Warning threshold (0.0 to 1.0)
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: f64,
}

fn default_max_tokens() -> usize {
    10000
}

fn default_warn_threshold() -> f64 {
    0.8
}

impl Default for CostLimiterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_tokens: default_max_tokens(),
            warn_threshold: default_warn_threshold(),
        }
    }
}

/// Tool allowlist configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAllowlistConfig {
    /// Enable tool allowlist
    #[serde(default)]
    pub enabled: bool,

    /// Allowed tool names
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}

impl Default for ToolAllowlistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_tools: Vec::new(),
        }
    }
}

impl GuardrailConfig {
    /// Build a GuardrailChain from configuration
    ///
    /// Only enabled guardrails are added to the chain.
    pub fn build_chain(&self) -> Result<Option<GuardrailChain>> {
        if !self.enabled {
            return Ok(None);
        }

        let mut chain = GuardrailChain::new();
        let mut count = 0;

        // Add content filter if enabled
        if self.content_filter.enabled {
            let mut filter = ContentFilter::new(self.content_filter.blocked_phrases.clone())
                .case_sensitive(self.content_filter.case_sensitive);

            // Add regex patterns
            for pattern in &self.content_filter.blocked_patterns {
                filter = filter.with_pattern(pattern)?;
            }

            chain = chain.with_guardrail(filter);
            count += 1;
        }

        // Add rate limiter if enabled
        if self.rate_limiter.enabled {
            let limiter = RateLimiter::new(
                self.rate_limiter.max_requests,
                Duration::from_secs(self.rate_limiter.window_seconds),
            );
            chain = chain.with_guardrail(limiter);
            count += 1;
        }

        // Add cost limiter if enabled
        if self.cost_limiter.enabled {
            let limiter = CostLimiter::new(self.cost_limiter.max_tokens)
                .warn_threshold(self.cost_limiter.warn_threshold);
            chain = chain.with_guardrail(limiter);
            count += 1;
        }

        // Add tool allowlist if enabled
        if self.tool_allowlist.enabled {
            let allowlist = ToolAllowlist::new(self.tool_allowlist.allowed_tools.clone());
            chain = chain.with_guardrail(allowlist);
            count += 1;
        }

        if count == 0 {
            Ok(None)
        } else {
            tracing::info!("Built guardrail chain with {} guardrails", count);
            Ok(Some(chain))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GuardrailConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_build_chain_disabled() {
        let config = GuardrailConfig::default();
        let chain = config.build_chain().unwrap();
        assert!(chain.is_none());
    }

    #[test]
    fn test_build_chain_with_guardrails() {
        let config = GuardrailConfig {
            enabled: true,
            content_filter: ContentFilterConfig {
                enabled: true,
                blocked_phrases: vec!["test".to_string()],
                ..Default::default()
            },
            rate_limiter: RateLimiterConfig {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let chain = config.build_chain().unwrap();
        assert!(chain.is_some());
        assert_eq!(chain.unwrap().len(), 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = GuardrailConfig {
            enabled: true,
            content_filter: ContentFilterConfig {
                enabled: true,
                blocked_phrases: vec!["bad".to_string()],
                blocked_patterns: vec![r"\d+".to_string()],
                case_sensitive: false,
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GuardrailConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content_filter.blocked_phrases.len(), 1);
    }
}

