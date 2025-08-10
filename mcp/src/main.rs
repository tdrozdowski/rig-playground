//! Binary entry for MCP HTTP server.
//! Exposes JSON-RPC endpoint at /mcp using rmcp streamable HTTP transport.

mod example_mcp;

use example_mcp::ExampleTools;
use rmcp::transport::StreamableHttpServerConfig;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init logging/tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting MCP HTTP (streamable) server...");

    let config = StreamableHttpServerConfig {
        sse_keep_alive: None,
        stateful_mode: true,
    };

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
