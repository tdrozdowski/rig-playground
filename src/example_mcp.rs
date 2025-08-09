use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, Json, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Counter>,
}

#[tool_router]
impl Counter {
    pub fn new() -> Self {
        Counter {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

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
}

#[tool_handler]
impl ServerHandler for Counter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple tool to count".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct CalculationRequest {
    a: i32,
    b: i32,
    operation: String,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
struct CalculationResult {
    result: i32,
    operation: String,
}

#[derive(Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Calculator {
            tool_router: Self::tool_router(),
        }
    }

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
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("A simple tool to perform calculations".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
