//! Parser for tool calls from LLM responses

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::AgentRuntimeError, Result};

/// A tool call extracted from an LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to call
    pub name: String,
    
    /// Parameters for the tool
    pub parameters: Value,
    
    /// Optional ID for tracking (from LLM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(name: String, parameters: Value, id: Option<String>) -> Self {
        Self {
            name,
            parameters,
            id,
        }
    }
}

/// Parse OpenAI function calls from response
///
/// OpenAI returns tool calls in this format:
/// ```json
/// {
///   "tool_calls": [{
///     "id": "call_abc",
///     "type": "function",
///     "function": {
///       "name": "calculator",
///       "arguments": "{\"a\": 5, \"b\": 3}"
///     }
///   }]
/// }
/// ```
pub fn parse_openai_tool_calls(response_content: &Value) -> Result<Vec<ToolCall>> {
    let tool_calls = response_content
        .get("tool_calls")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AgentRuntimeError::parse("No tool_calls array in response"))?;

    let mut parsed_calls = Vec::new();

    for call in tool_calls {
        let id = call.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        let function = call
            .get("function")
            .ok_or_else(|| AgentRuntimeError::parse("Missing function field"))?;

        let name = function
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentRuntimeError::parse("Missing function name"))?
            .to_string();

        let arguments_str = function
            .get("arguments")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentRuntimeError::parse("Missing function arguments"))?;

        let parameters: Value = serde_json::from_str(arguments_str)
            .map_err(|e| AgentRuntimeError::parse(format!("Invalid JSON arguments: {}", e)))?;

        parsed_calls.push(ToolCall {
            name,
            parameters,
            id,
        });
    }

    Ok(parsed_calls)
}

/// Parse Anthropic tool uses from response
///
/// Anthropic returns content blocks:
/// ```json
/// {
///   "content": [{
///     "type": "tool_use",
///     "id": "toolu_abc",
///     "name": "calculator",
///     "input": {"a": 5, "b": 3}
///   }]
/// }
/// ```
pub fn parse_anthropic_tool_calls(response_content: &Value) -> Result<Vec<ToolCall>> {
    let content = response_content
        .get("content")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AgentRuntimeError::parse("No content array in response"))?;

    let mut parsed_calls = Vec::new();

    for block in content {
        let block_type = block.get("type").and_then(|v| v.as_str());
        
        if block_type != Some("tool_use") {
            continue; // Skip non-tool blocks
        }

        let id = block.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());

        let name = block
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentRuntimeError::parse("Missing tool name"))?
            .to_string();

        let parameters = block
            .get("input")
            .ok_or_else(|| AgentRuntimeError::parse("Missing tool input"))?
            .clone();

        parsed_calls.push(ToolCall {
            name,
            parameters,
            id,
        });
    }

    Ok(parsed_calls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_openai_tool_call() {
        let response = json!({
            "tool_calls": [{
                "id": "call_123",
                "type": "function",
                "function": {
                    "name": "calculator",
                    "arguments": "{\"operation\": \"add\", \"a\": 5, \"b\": 3}"
                }
            }]
        });

        let calls = parse_openai_tool_calls(&response).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "calculator");
        assert_eq!(calls[0].parameters["operation"], "add");
        assert_eq!(calls[0].id.as_ref().unwrap(), "call_123");
    }

    #[test]
    fn test_parse_anthropic_tool_call() {
        let response = json!({
            "content": [{
                "type": "tool_use",
                "id": "toolu_abc",
                "name": "calculator",
                "input": {
                    "operation": "multiply",
                    "a": 7,
                    "b": 6
                }
            }]
        });

        let calls = parse_anthropic_tool_calls(&response).unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "calculator");
        assert_eq!(calls[0].parameters["operation"], "multiply");
        assert_eq!(calls[0].id.as_ref().unwrap(), "toolu_abc");
    }

    #[test]
    fn test_parse_multiple_tool_calls() {
        let response = json!({
            "tool_calls": [
                {
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "calculator",
                        "arguments": "{\"operation\": \"add\", \"a\": 1, \"b\": 2}"
                    }
                },
                {
                    "id": "call_2",
                    "type": "function",
                    "function": {
                        "name": "echo",
                        "arguments": "{\"text\": \"hello\"}"
                    }
                }
            ]
        });

        let calls = parse_openai_tool_calls(&response).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "calculator");
        assert_eq!(calls[1].name, "echo");
    }

    #[test]
    fn test_parse_no_tool_calls() {
        let response = json!({
            "message": "Just a regular response"
        });

        let result = parse_openai_tool_calls(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_arguments() {
        let response = json!({
            "tool_calls": [{
                "id": "call_123",
                "type": "function",
                "function": {
                    "name": "calculator",
                    "arguments": "invalid json {"
                }
            }]
        });

        let result = parse_openai_tool_calls(&response);
        assert!(result.is_err());
    }
}

