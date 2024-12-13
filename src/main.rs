//! Simple server for generating greeting SVG

mod config;
mod counter;
mod handler;
mod svg;
mod utils;

use anyhow::Result;
use axum::routing::get;
use macro_toolset::init_tracing_simple;
use miku_server_timing::ServerTimingLayer;
use tokio::net::TcpListener;
use tower_http::compression::CompressionLayer;

// Mimalloc
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_simple!();

    let config = config::Config::parse()?;

    tracing::info!("{:?}", config);

    counter::Counter::init(&config).await;

    let tcp_listener = TcpListener::bind(config.listen).await?;

    // Main Server
    let _ = axum::serve(
        tcp_listener,
        axum::Router::new()
            .route("/greeting/:id", get(handler::axum_greeting))
            .layer(CompressionLayer::new())
            .layer(ServerTimingLayer::new(utils::VERSION))
            .fallback(handler::fallback),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await;

    post_task().await?;

    Ok(())
}

/// axum graceful shutdown signal
async fn shutdown_signal() {
    #[cfg(unix)]
    let hangup = async {
        use tokio::signal::unix::{signal, SignalKind};
        signal(SignalKind::hangup()).unwrap().recv().await;
    };

    #[cfg(not(unix))]
    let hangup = std::future::pending::<()>();

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = hangup => {
            tracing::info!("Received SIGHUP");
        }
    }
}

/// Post task after the server stopped
async fn post_task() -> Result<()> {
    counter::Counter::persist_all().await?;

    Ok(())
}
