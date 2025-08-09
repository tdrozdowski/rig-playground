//!
//! Binary entry for Rig Playground MCP servers.
//!
//! This binary demonstrates how to run example MCP tools (see [`crate::example_mcp`])
//! over multiple transports using the `rmcp` crate:
//! - HTTP JSON-RPC at `/mcp` with streamable responses and stateful sessions (see [`main_http`]).
//! - Server-Sent Events (SSE) at `/sse` with POST `/message` (see [`main_sse`]).
//! - stdio transport for embedding into a host process (see [`counter_tool`]).
//!
//! Configuration:
//! - Set `RUST_LOG` to control logging (e.g., `info`, `debug`).
//! - Default bind address is `127.0.0.1:8000`.
//!
//! Switching modes:
//! - Edit [`main`] to call the desired function: [`main_http`], [`main_sse`], or [`counter_tool`].
//!
pub mod example_mcp;

use crate::example_mcp::ExampleTools;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
/// Entry point for the example servers.
///
/// Environment variables
/// - RUST_LOG: set the log level for tracing (e.g., "info", "debug").
///   If unset, defaults to "debug" in this example.
///
/// Deployment notes
/// - The HTTP streamable server binds to BIND_ADDRESS (default: 127.0.0.1:8000)
///   and exposes a JSON-RPC MCP endpoint at `/mcp`.
/// - The SSE server variant (if enabled) binds to the same address and uses
///   `/sse` for server-sent events and `/message` for POST messages.
///
/// Switching modes
/// - By default, this binary runs the HTTP streamable server (main_http).
/// - To run the SSE server or the stdio tool server, uncomment the respective
///   lines below.
async fn main() -> anyhow::Result<()> {
    //counter_tool().await
    //main_sse().await
    main_http().await
}

async fn main_http() -> anyhow::Result<()> {
    // Init logging/tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting MCP HTTP (streamable) server...");

    // Cancellation for graceful shutdown
    let config = StreamableHttpServerConfig {
        sse_keep_alive: None,
        stateful_mode: true,
    };
    // Create the HTTP streamable service and its Axum router
    // Exposes a single JSON-RPC endpoint at /mcp that supports streamable responses
    let service = StreamableHttpService::new(
        || Ok(ExampleTools::new()),
        LocalSessionManager::default().into(),
        config,
    );
    let router = axum::Router::new().nest_service("/mcp", service);

    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await;
    Ok(())
}
