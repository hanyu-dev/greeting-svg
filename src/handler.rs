//! Request Handlers

use std::{borrow::Cow, net::IpAddr};

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Path, Request},
    http::{HeaderName, HeaderValue, Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};

use crate::{counter::Counter, svg, utils::Queries};

#[inline]
#[tracing::instrument]
/// Greeting router
pub(crate) async fn axum_greeting_no_path(request: Request) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<false>(None, request).await {
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

    match greeting::<false>(Some(id), request).await {
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
/// Moe counter router
pub(crate) async fn axum_moe_counter_no_path(request: Request) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<true>(None, request).await {
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
/// Moe counter router
pub(crate) async fn axum_moe_counter(
    Path(id): Path<Cow<'static, str>>,
    request: Request,
) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<true>(Some(id), request).await {
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
async fn greeting<const FORCE_MOE_COUNTER: bool>(
    id: Option<Cow<'_, str>>,
    request: Request,
) -> Result<Response> {
    let queries = Queries::try_parse_uri(request.uri());

    let id = id
        .as_ref()
        .or_else(|| queries.get("id"))
        .or_else(|| queries.get("key"))
        .take_if(|id| !id.is_empty())
        .map(|id| id.trim_start_matches("@"))
        .context("Invalid id, empty or not given.")?;

    let access_count = {
        let remote_ip: Option<IpAddr> = request
            .headers()
            .get(X_FORWARDED_FOR)
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.parse().ok());
        let access_key = queries.get("access_key");

        if request.method() == Method::DELETE {
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
        }
    };

    // * Greeting type, can be moe-counter, or default one.
    let greeting_type = queries.get("type").map(AsRef::as_ref);

    let mut content = match greeting_type {
        Some("linux-do-card") => {
            svg::linux_do_card::LinuxDoCardImpl::new(
                queries
                    .get("user")
                    .map(|user| user.trim_start_matches('@'))
                    .or(Some(id)),
                queries
                    .get("timezone")
                    .and_then(|tz| tz.parse().ok())
                    .unwrap_or(chrono_tz::Tz::Asia__Shanghai),
            )
            .set_custom_bio(queries.get("note"))
            .await
            .generate(access_count.unwrap_or_default())
            .await
        }
        Some("moe-counter") => svg::moe_counter::MoeCounterImpl::from_queries(&queries)
            .generate(access_count.unwrap_or_default()),
        _ if FORCE_MOE_COUNTER => svg::moe_counter::MoeCounterImpl::from_queries(&queries)
            .generate(access_count.unwrap_or_default()),
        _ => {
            // * Custom Timezone, default to Asia/Shanghai
            let tz = queries
                .get("timezone")
                .and_then(|tz| tz.parse().ok())
                .unwrap_or(chrono_tz::Tz::Asia__Shanghai);

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

#[tracing::instrument]
#[inline]
pub(crate) async fn not_found(_request: Request) -> Response {
    StatusCode::NOT_FOUND.into_response()
}
