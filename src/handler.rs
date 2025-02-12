//! Request Handlers

use std::{borrow::Cow, net::IpAddr};

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Path, Request},
    http::{header::CONTENT_TYPE, HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{counter::Counter, svg, utils::Queries};

#[inline]
#[tracing::instrument]
/// Greeting router
pub(crate) async fn axum_greeting_no_path(request: Request) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting(None, request).await {
        Ok(greeting) => Ok(greeting),
        Err(error) => match error.downcast::<StatusCode>() {
            Ok(status_code) => Err(status_code),
            Err(error) => {
                tracing::error!("{:?}", error);
                Err(StatusCode::BAD_REQUEST)
            }
        },
    }
}

#[inline]
#[tracing::instrument]
/// Greeting router
pub(crate) async fn axum_greeting(
    Path(id): Path<Cow<'static, str>>,
    request: Request,
) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting(Some(id), request).await {
        Ok(greeting) => Ok(greeting),
        Err(error) => match error.downcast::<StatusCode>() {
            Ok(status_code) => Err(status_code),
            Err(error) => {
                tracing::error!("{:?}", error);
                Err(StatusCode::BAD_REQUEST)
            }
        },
    }
}

const X_FORWARDED_FOR: HeaderName = HeaderName::from_static("x-forwarded-for");

#[inline]
async fn greeting(id: Option<Cow<'_, str>>, request: Request) -> Result<Response> {
    let queries = Queries::try_parse_uri(request.uri());

    let id = id
        .as_ref()
        .or_else(|| queries.get("id"))
        .or_else(|| queries.get("key"))
        .take_if(|id| !id.is_empty())
        .map(|id| id.trim_start_matches("@"))
        .context("Invalid id, empty or not given.")?;

    let remote_ip: Option<IpAddr> = request
        .headers()
        .get(X_FORWARDED_FOR)
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.parse().ok());

    let access_key = queries.get("access_key");

    let access_count = if request.method() == Method::DELETE {
        Counter::delete(id, access_key, remote_ip).await?;

        return Ok(StatusCode::OK.into_response());
    } else {
        Counter::fetch_add(
            id,
            access_key,
            queries.get("debug").is_some_and(|d| d == "true"),
            remote_ip,
        )
        .await
    };

    // * Custom Timezone, default to Asia/Shanghai
    let tz = queries
        .get("timezone")
        .and_then(|tz| tz.parse().ok())
        .unwrap_or(chrono_tz::Tz::Asia__Shanghai);

    // * Greeting type, can be moe-counter, or default one.
    let greeting_type = queries.get("type").map(AsRef::as_ref);

    let mut content = match greeting_type {
        Some("moe-counter") => svg::moe_counter::MoeCounterImpl {
            theme: queries
                .get("theme")
                .map(AsRef::as_ref)
                .unwrap_or("moebooru"),
            padding: queries
                .get("padding")
                .and_then(|padding| padding.parse().ok())
                .unwrap_or(7),
            offset: queries
                .get("offset")
                .and_then(|offset| offset.parse().ok())
                .unwrap_or(0.0),
            align: queries.get("align").map(AsRef::as_ref).unwrap_or("top"),
            scale: queries
                .get("scale")
                .and_then(|scale| scale.parse().ok())
                .unwrap_or(1.0),
            pixelated: queries
                .get("pixelated")
                .map(|pixelated| pixelated == "1" || pixelated == "true")
                .unwrap_or(false),
            darkmode: queries
                .get("darkmode")
                .map(|darkmode| darkmode == "1" || darkmode == "true"),
            prefix: queries.get("prefix").and_then(|prefix| prefix.parse().ok()),
        }
        .generate(access_count.unwrap_or_default()),
        _ => {
            svg::GeneralImpl {
                tz,
                access_count,
                bg_type: queries
                    .get("bg_type")
                    .map(|bg_type| bg_type.parse().unwrap())
                    .unwrap_or_default(),
                note: queries.get("note"),
            }
            .generate()
            .await
        }
    }
    .into_bytes();

    // ! Avoid unnecessary allocation
    content.shrink_to_fit();

    Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"))
        .body(Body::from(bytes::Bytes::from(content)))
        .map_err(Into::into)
}

#[inline]
pub(crate) async fn not_found(_request: Request) -> Response {
    StatusCode::NOT_FOUND.into_response()
}

#[tracing::instrument]
#[inline]
// TODO: Shutdown connection immediately
pub(crate) async fn fallback(request: Request) -> Response {
    tracing::warn!("No available handler");

    StatusCode::NOT_FOUND.into_response()
}
