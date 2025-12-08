//! JSON Schema generation for tools

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON Schema for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolSchema {
    /// Type (usually "object" for tool parameters)
    #[serde(rename = "type")]
    pub schema_type: String,
    
    /// Properties of the object
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Value>,
    
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    
    /// Description of the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ToolSchema {
    /// Create a new tool schema
    pub fn new() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            description: None,
        }
    }

    /// Set properties
    pub fn with_properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set required fields
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Set description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Convert to OpenAI function format
    pub fn to_openai_function(&self, name: &str, description: &str) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": name,
                "description": description,
                "parameters": self,
            }
        })
    }

    /// Convert to Anthropic tool format
    pub fn to_anthropic_tool(&self, name: &str, description: &str) -> Value {
        serde_json::json!({
            "name": name,
            "description": description,
            "input_schema": self,
        })
    }
}

impl Default for ToolSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a simple property schema
pub fn property(type_name: &str, description: &str) -> Value {
    serde_json::json!({
        "type": type_name,
        "description": description,
    })
}

/// Helper to create an enum property schema
pub fn enum_property(type_name: &str, description: &str, values: Vec<String>) -> Value {
    serde_json::json!({
        "type": type_name,
        "description": description,
        "enum": values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = ToolSchema::new()
            .with_description("Test schema")
            .with_properties(serde_json::json!({
                "name": property("string", "The name"),
                "age": property("number", "The age"),
            }))
            .with_required(vec!["name".to_string()]);

        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_some());
        assert!(schema.required.is_some());
    }

    #[test]
    fn test_openai_format() {
        let schema = ToolSchema::new()
            .with_properties(serde_json::json!({
                "query": property("string", "Search query"),
            }))
            .with_required(vec!["query".to_string()]);

        let openai_func = schema.to_openai_function("search", "Search the web");
        
        assert_eq!(openai_func["type"], "function");
        assert_eq!(openai_func["function"]["name"], "search");
        assert_eq!(openai_func["function"]["description"], "Search the web");
    }

    #[test]
    fn test_anthropic_format() {
        let schema = ToolSchema::new()
            .with_properties(serde_json::json!({
                "city": property("string", "City name"),
            }));

        let anthropic_tool = schema.to_anthropic_tool("weather", "Get weather");
        
        assert_eq!(anthropic_tool["name"], "weather");
        assert_eq!(anthropic_tool["description"], "Get weather");
        assert!(anthropic_tool["input_schema"].is_object());
    }

    #[test]
    fn test_property_helper() {
        let prop = property("string", "A string field");
        assert_eq!(prop["type"], "string");
        assert_eq!(prop["description"], "A string field");
    }

    #[test]
    fn test_enum_property() {
        let prop = enum_property("string", "Operation", vec!["add".to_string(), "subtract".to_string()]);
        assert_eq!(prop["type"], "string");
        assert_eq!(prop["enum"][0], "add");
        assert_eq!(prop["enum"][1], "subtract");
    }
}

