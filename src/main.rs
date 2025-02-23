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
use tokio::{net::TcpListener, task::JoinSet};
use tower_http::compression::CompressionLayer;

// Mimalloc
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_simple!();

    let config = config::Config::parse()?;

    tracing::info!("{:#?}", config);

    counter::Counter::init(&config).await;

    let service = axum::Router::new()
        .route(
            "/greeting",
            get(handler::axum_greeting_no_path).delete(handler::axum_greeting_no_path),
        )
        .route(
            "/greeting/{id}",
            get(handler::axum_greeting).delete(handler::axum_greeting),
        )
        .route(
            "/moe-counter",
            get(handler::axum_moe_counter_no_path).delete(handler::axum_moe_counter_no_path),
        )
        .route("/moe-counter/", get(handler::axum_moe_counter_index))
        .route(
            "/moe-counter/{id}",
            get(handler::axum_moe_counter).delete(handler::axum_moe_counter),
        )
        .route(
            "/linux-do-card",
            get(handler::axum_linux_do_card_no_path).delete(handler::axum_linux_do_card_no_path),
        )
        .route("/linux-do-card/", get(handler::axum_linux_do_card_index))
        .route(
            "/linux-do-card/{id}",
            get(handler::axum_linux_do_card).delete(handler::axum_linux_do_card),
        )
        .layer(CompressionLayer::new())
        .layer(ServerTimingLayer::new(env!("CARGO_PKG_NAME")).with_description(utils::VERSION))
        .fallback(handler::not_found);

    let mut server_handlers = JoinSet::new();

    for listen in config.listen {
        // Main Server
        match listen {
            config::ListenAddr::SocketAddr(socket_addr) => {
                let tcp_listener = TcpListener::bind(socket_addr).await?;
                let service = service.clone();
                server_handlers.spawn(async move {
                    (
                        socket_addr.to_string(),
                        axum::serve(tcp_listener, service)
                            .with_graceful_shutdown(shutdown_signal())
                            .await,
                    )
                });
            }
            #[cfg(not(unix))]
            config::ListenAddr::Unix(unix_path) => {
                panic!("Unix socket is not supported on this platform")
            }
            #[cfg(unix)]
            config::ListenAddr::Unix(unix_path) => {
                use tokio::net::UnixListener;

                let unix_listener = UnixListener::bind(&unix_path)?;
                let service = service.clone();
                server_handlers.spawn(async move {
                    (
                        format!("unix:{unix_path}"),
                        axum::serve(unix_listener, service)
                            .with_graceful_shutdown(shutdown_signal())
                            .await,
                    )
                });
            }
        }
    }

    for (idx, result) in server_handlers.join_all().await {
        tracing::info!("Server shutdown result for [{idx}]: {result:?}");
    }

    post_task().await?;

    Ok(())
}

/// axum graceful shutdown signal
async fn shutdown_signal() {
    #[cfg(unix)]
    let hangup = async {
        use tokio::signal::unix::{SignalKind, signal};
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
