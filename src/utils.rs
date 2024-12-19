//! Utilities

pub(crate) mod ammonia;

use std::{borrow::Cow, collections::HashMap, net::IpAddr};

use axum::http::Uri;
use macro_toolset::wrapper;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::config::{CONF_ACCESS_KEY, CONF_CIDR_WHITELIST};

/// The version of the crate.
pub(crate) static VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));

// /// The version of the server.
// pub(crate) static BUILD_TIME: &str = include_str!(concat!(env!("OUT_DIR"),
// "/BUILD_TIME"));

wrapper!(pub(crate) Queries<'q>(HashMap<Cow<'q, str>, Cow<'q, str>, foldhash::fast::RandomState>), derive(Debug, Default));

impl<'q> Queries<'q> {
    #[inline]
    /// Parse query string from URI
    pub(crate) fn try_parse_uri(uri: &'q Uri) -> Self {
        uri.query().map(Self::try_parse).unwrap_or_default()
    }

    #[inline]
    /// Parse query string
    pub(crate) fn try_parse(query: &'q str) -> Self {
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
            .map(|(k, v)| {
                (
                    k.decode().into_string_lossy(),
                    v.decode().into_string_lossy(),
                )
            })
            .collect::<HashMap<_, _, _>>()
            .into()
    }
}

#[inline]
/// Check request auth with `access_key`
pub(crate) fn auth(access_key: Option<impl AsRef<str>>, remote_ip: Option<IpAddr>) -> bool {
    remote_ip.is_some_and(|remote_ip| {
        CONF_CIDR_WHITELIST
            .par_iter()
            .any(|cidr_whitelist| cidr_whitelist.contains(&remote_ip))
    }) || CONF_ACCESS_KEY
        .get()
        .zip(access_key)
        .is_some_and(|(desired_access_key, access_key)| {
            desired_access_key.load().as_str() == access_key.as_ref()
        })
}
