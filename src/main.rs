//! Simple server for generating greeting SVG

mod config;
mod counter;
mod svg;

use std::{borrow::Cow, collections::HashMap};

use anyhow::Result;
use axum::{
    extract::{Path, Request},
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use macro_toolset::init_tracing_simple;
use tokio::net::TcpListener;

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
            .route("/greeting/:id", get(axum_greeting))
            .fallback(fallback),
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

#[inline]
#[tracing::instrument]
async fn axum_greeting(Path(id): Path<String>, request: Request) -> Result<Response<String>, StatusCode> {
    tracing::debug!("Accepted request.");

    if id.len() > 256 {
        tracing::warn!("ID too long");

        return Err(StatusCode::BAD_REQUEST);
    }

    match greeting(id, request).await {
        Ok(greeting) => Ok(greeting),
        Err(error) => {
            tracing::error!("{:?}", error);

            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[inline]
async fn greeting(id: String, request: Request) -> Result<Response<String>> {
    let queries = request.uri().query().map(parse_query).unwrap_or_default();

    let access_key = queries.get("access_key");
    let access_count = counter::Counter::fetch_add(&id, access_key).await;

    // * Custom Timezone, default to Asia/Shanghai
    let tz = queries
        .get("timezone")
        .and_then(|tz| tz.parse().ok())
        .unwrap_or(chrono_tz::Tz::Asia__Shanghai);

    // * Note
    let note = queries.get("note");

    Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"))
        .body(svg::Greeting { tz, access_count, note }.generate().await)
        .map_err(Into::into)
}

// TODO: Shutdown connection immediately
async fn fallback(request: Request) -> Response {
    tracing::warn!(?request, "No available handler");

    StatusCode::NOT_FOUND.into_response()
}

#[inline]
fn parse_query(query: &str) -> HashMap<Cow<str>, Cow<str>, foldhash::fast::RandomState> {
    use fluent_uri::encoding::{encoder::IQuery, EStr};

    EStr::<IQuery>::new(query)
        .unwrap_or_else(|| {
            tracing::warn!("Failed to parse query: {:?}", query);

            EStr::EMPTY
        })
        .split('&')
        .map(|pair| {
            pair.split_once('=').unwrap_or_else(|| {
                tracing::warn!("Failed to split query pair: {:?}", pair);

                (pair, EStr::EMPTY)
            })
        })
        .map(|(k, v)| (k.decode().into_string_lossy(), v.decode().into_string_lossy()))
        .collect()
}
