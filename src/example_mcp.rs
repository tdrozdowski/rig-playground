//! Example MCP tools used by this crate.
//!
//! This module defines two simple MCP tools to demonstrate how to build a
//! server with the `rmcp` crate:
//! - `Counter`: a stateful tool that increments an in-memory counter.
//! - `Calculator`: a stateless tool that performs basic arithmetic.
//!
//! Both tools implement `ServerHandler` via macros provided by `rmcp`, and can be
//! served over stdio, SSE, or an HTTP JSON-RPC endpoint as shown in `main.rs`.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, Json, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::info;

/// Combined MCP tools: a stateful counter and a stateless calculator under a single router/handler.
#[derive(Clone)]
pub struct ExampleTools {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ExampleTools {
    /// Construct a new combined tools handler with counter initialized to 0.
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    /// Increment the counter by 1 and return the previous value as text content.
    ///
    /// Returns
    /// - `Ok(CallToolResult)` with a single text content containing the previous
    ///   counter value.
    /// - `Err(ErrorData)` if internal state cannot be accessed (mutex poisoned).
    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, ErrorData> {
        let mut guard = self
            .counter
            .lock()
            .map_err(|_| ErrorData::internal_error("counter mutex poisoned".to_string(), None))?;
        let counter = *guard;
        *guard += 1;
        info!("Counter is now {}", counter);
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    /// Perform a calculation based on the provided parameters.
    ///
    /// Supported operations
    /// - `"add"`: returns `a + b`
    /// - `"multiply"`: returns `a * b`
    ///
    /// Errors
    /// - Returns `Err(String)` for unknown operations.
    #[tool(name = "calculate", description = "Perform a calculation")]
    async fn calculate(
        &self,
        params: Parameters<CalculationRequest>,
    ) -> Result<Json<CalculationResult>, String> {
        let result = match params.0.operation.as_str() {
            "add" => params.0.a + params.0.b,
            "multiply" => params.0.a * params.0.b,
            _ => return Err("Unknown operation".to_string()),
        };

        Ok(Json(CalculationResult {
            result,
            operation: params.0.operation,
        }))
    }
}

#[tool_handler]
impl ServerHandler for ExampleTools {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Combined tools: counter and calculator".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Input for the `calculate` tool.
///
/// Fields
/// - `a`: first operand
/// - `b`: second operand
/// - `operation`: the operation to perform; supported values are `"add"` and `"multiply"`.
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct CalculationRequest {
    a: i32,
    b: i32,
    operation: String,
}

/// Output of the `calculate` tool.
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct CalculationResult {
    result: i32,
    operation: String,
}
