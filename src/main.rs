pub mod example_mcp;

use crate::example_mcp::{Calculator, Counter};
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

    // Register the Calculator service
    let server_ct = sse_server.with_service_directly(Counter::new);

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

    // Create an instance of our counter router
    let service = Counter::new().serve(stdio()).await.inspect_err(|e| {
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
    let ct = CancellationToken::new();
    let child_ct = ct.child_token();
    let config = StreamableHttpServerConfig {
        sse_keep_alive: Some(std::time::Duration::from_secs(10)),
        stateful_mode: true,
    };
    // Create the HTTP streamable service and its Axum router
    // Exposes a single JSON-RPC endpoint at /mcp that supports streamable responses
    let service = StreamableHttpService::new(
        || Ok(Counter::new()),
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
