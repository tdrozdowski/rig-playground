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
use rmcp::service::ServiceExt;
use rmcp::transport::sse_server::{SseServer, SseServerConfig};
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::transport::{StreamableHttpServerConfig, stdio};
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

const BIND_ADDRESS: &str = "127.0.0.1:8000";

async fn main_sse() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting MCP server...");

    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: CancellationToken::new(),
        sse_keep_alive: None,
    };

    // Create the server and router
    let (sse_server, router) = SseServer::new(config);

    // Start the HTTP server with the router
    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;
    let ct = sse_server.config.ct.child_token();

    info!("HTTP server binding to {}", sse_server.config.bind);

    let server_handle = tokio::spawn(async move {
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            ct.cancelled().await;
            info!("HTTP server shutting down");
        });

        if let Err(e) = server.await {
            tracing::error!(error = %e, "HTTP server error");
        }
    });

    // Register the combined tools service
    let server_ct = sse_server.with_service_directly(ExampleTools::new);

    info!("MCP server ready on {}", BIND_ADDRESS);

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");
    server_ct.cancel();

    // Wait for server to shutdown
    let _ = server_handle.await;

    Ok(())
}

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

async fn counter_tool() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our combined tools router
    let service = ExampleTools::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}

async fn calculator_tool() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // attempt sse here
    Ok(())
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
