//! Helper functions for agent integration

/// Placeholder for OpenAI tool sending
pub async fn send_openai_with_tools(
    _messages: Vec<crate::Message>,
    _tools: Vec<serde_json::Value>,
) -> crate::Result<serde_json::Value> {
    // This will be properly implemented when we add full tool support
    Ok(serde_json::json!({}))
}

/// Placeholder for Anthropic tool sending
pub async fn send_anthropic_with_tools(
    _messages: Vec<crate::Message>,
    _tools: Vec<serde_json::Value>,
) -> crate::Result<serde_json::Value> {
    // This will be properly implemented when we add full tool support
    Ok(serde_json::json!({}))
}

