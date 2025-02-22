//! Request Handlers

use std::{borrow::Cow, net::IpAddr};

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Path, Request},
    http::{
        HeaderName, HeaderValue, Method, StatusCode,
        header::{CONTENT_TYPE, REFERER},
    },
    response::{IntoResponse, Response},
};

use crate::{counter::Counter, svg, utils::Queries};

#[inline]
#[tracing::instrument]
/// Greeting router
pub(crate) async fn axum_greeting_no_path(request: Request) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<false, false>(None, request).await {
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

    match greeting::<false, false>(Some(id), request).await {
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

    match greeting::<true, false>(None, request).await {
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
/// Linux.do card router
pub(crate) async fn axum_moe_counter_index(request: Request) -> Response {
    tracing::debug!("Accepted request.");

    let response = r#"
<html lang="zh" data-bs-theme="auto">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>Moe-counter</title>
    </head>
    <body>
        <h1>Moe-counter</h1>
        <h2>使用方法</h2>
        <p>只需要添加参数 <i>type=moe-counter</i>, 其余参数和原版基本一致.</p>
        <p>例如: https://greeting.app.acfun.win/Hantong?type=moe-counter</p>
        <h2>示例图片</h2>
        <img src="https://greeting.app.acfun.win/Hantong?type=moe-counter" height="220">
    </body>
</html>
    "#;

    Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
        .body(Body::from(bytes::Bytes::from(response)))
        .unwrap()
}

#[inline]
#[tracing::instrument]
/// Moe counter router
pub(crate) async fn axum_moe_counter(
    Path(id): Path<Cow<'static, str>>,
    request: Request,
) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<true, false>(Some(id), request).await {
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
pub(crate) async fn axum_linux_do_card_no_path(request: Request) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<false, true>(None, request).await {
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
/// Linux.do card router
pub(crate) async fn axum_linux_do_card_index(request: Request) -> Response {
    tracing::debug!("Accepted request.");

    let response = r#"
<html lang="zh" data-bs-theme="auto">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <title>Linux.do Card</title>
    </head>
    <body>
        <h1>Linux.do Card</h1>
        <p>数据是实时的, 但是为了访问速度和不触发反爬机制, 会有 300 秒的缓存.</p>
        <h2>使用方法</h2>
        <p>URL 格式: <i>greeting.app.acfun.win/{你的 Linux.do 用户名}?type=linux-do-card</i>. 注意应尽量区分大小写, 同时不要和昵称混淆.</p>
        <p>可选参数: </p>
        <ul>
            <li><em>note</em> 自定义 Bio, 否则读取 Linux.do 中设定的 Bio</li>
            <li><em>timezone</em> 自定义时区, 默认为 Asia/Shanghai, 可选值参见 chrono-tz 库</li>
        </ul>
        <p>例如: https://greeting.app.acfun.win/Hantong?type=linux-do-card&amp;note=%E6%88%91%E7%9A%84%E5%8D%9A%E5%AE%A2%3A%20https%3A%2F%2Facfun.win</p>
        <h2>示例图片</h2>
        <img src="https://greeting.app.acfun.win/Hantong?type=linux-do-card&amp;note=Hi%20from%20Index%21" height="220">
    </body>
</html>
    "#;

    Response::builder()
        .header(CONTENT_TYPE, HeaderValue::from_static("text/html"))
        .body(Body::from(bytes::Bytes::from(response)))
        .unwrap()
}

#[inline]
#[tracing::instrument]
/// Moe counter router
pub(crate) async fn axum_linux_do_card(
    Path(id): Path<Cow<'static, str>>,
    request: Request,
) -> Result<Response, StatusCode> {
    tracing::debug!("Accepted request.");

    match greeting::<false, true>(Some(id), request).await {
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
async fn greeting<const FORCE_MOE_COUNTER: bool, const FORCE_LINUX_DO_CARD: bool>(
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
                id,
                queries
                    .get("timezone")
                    .and_then(|tz| tz.parse().ok())
                    .unwrap_or(chrono_tz::Tz::Asia__Shanghai),
                request
                    .headers()
                    .get(REFERER)
                    .is_some_and(|r| r.as_bytes().starts_with(b"https://linux.do/")),
            )
            .set_custom_bio(queries.get("note"))
            .await
            .generate(access_count)
            .await
        }
        _ if FORCE_LINUX_DO_CARD => {
            svg::linux_do_card::LinuxDoCardImpl::new(
                id,
                queries
                    .get("timezone")
                    .and_then(|tz| tz.parse().ok())
                    .unwrap_or(chrono_tz::Tz::Asia__Shanghai),
                request
                    .headers()
                    .get(REFERER)
                    .is_some_and(|r| r.as_bytes().starts_with(b"https://linux.do/")),
            )
            .set_custom_bio(queries.get("note"))
            .await
            .generate(access_count)
            .await
        }
        Some("moe-counter") => svg::moe_counter::MoeCounterImpl::from_queries(&queries)
            .generate(access_count.unwrap_or_default()),
        _ if FORCE_MOE_COUNTER => svg::moe_counter::MoeCounterImpl::from_queries(&queries)
            .generate(access_count.unwrap_or_default()),
        _ => {
            svg::GeneralImpl {
                tz: queries
                    .get("timezone")
                    .and_then(|tz| tz.parse().ok())
                    .unwrap_or(chrono_tz::Tz::Asia__Shanghai),
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
