//! Calculator tool for basic math operations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{schema::property, Result, Tool, ToolResult, ToolSchema};

/// Calculator tool for performing basic arithmetic
pub struct CalculatorTool;

#[derive(Debug, Deserialize, Serialize)]
struct CalculatorParams {
    operation: String,
    a: f64,
    b: f64,
}

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic arithmetic operations (add, subtract, multiply, divide)"
    }

    fn parameters_schema(&self) -> ToolSchema {
        ToolSchema::new()
            .with_description("Parameters for calculator")
            .with_properties(serde_json::json!({
                "operation": {
                    "type": "string",
                    "description": "The operation to perform",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "a": property("number", "The first number"),
                "b": property("number", "The second number"),
            }))
            .with_required(vec!["operation".to_string(), "a".to_string(), "b".to_string()])
    }

    async fn execute(&self, params: Value) -> Result<ToolResult> {
        let params: CalculatorParams = serde_json::from_value(params)
            .map_err(|e| crate::ToolError::invalid_params(e.to_string()))?;

        let result = match params.operation.as_str() {
            "add" => params.a + params.b,
            "subtract" => params.a - params.b,
            "multiply" => params.a * params.b,
            "divide" => {
                if params.b == 0.0 {
                    return Ok(ToolResult::error("Division by zero"));
                }
                params.a / params.b
            }
            op => {
                return Ok(ToolResult::error(format!("Unknown operation: {}", op)));
            }
        };

        Ok(ToolResult::success(serde_json::json!({
            "operation": params.operation,
            "a": params.a,
            "b": params.b,
            "result": result,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_calculator_add() {
        let calc = CalculatorTool;
        let params = serde_json::json!({
            "operation": "add",
            "a": 5.0,
            "b": 3.0,
        });

        let result = calc.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data.unwrap()["result"], 8.0);
    }

    #[tokio::test]
    async fn test_calculator_divide_by_zero() {
        let calc = CalculatorTool;
        let params = serde_json::json!({
            "operation": "divide",
            "a": 10.0,
            "b": 0.0,
        });

        let result = calc.execute(params).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_calculator_invalid_operation() {
        let calc = CalculatorTool;
        let params = serde_json::json!({
            "operation": "modulo",
            "a": 10.0,
            "b": 3.0,
        });

        let result = calc.execute(params).await.unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_calculator_schema() {
        let calc = CalculatorTool;
        let schema = calc.parameters_schema();
        
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_some());
        assert_eq!(schema.required.as_ref().unwrap().len(), 3);
    }
}

