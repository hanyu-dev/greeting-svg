//! Linux.do cards, data structures

use std::{sync::Arc, time::Instant};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default)]
/// Just wrapper over [`User`] and [`UserSummary`]
pub(super) struct UserInfo {
    pub created: Option<Instant>,
    pub user: User,
    pub user_summary: UserSummary,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Deserialize)]
#[serde(default)]
#[repr(transparent)]
/// <https://linux.do/u/{$username}/summary.json>
pub(super) struct UserSummaryAll {
    /// 用户信息
    pub user_summary: UserSummary,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Deserialize)]
#[serde(default)]
/// <https://linux.do/u/{$username}/summary.json> -> `user_summary`
pub(super) struct UserSummary {
    /// 已送出赞
    pub likes_given: u64,

    /// 已收到赞
    pub likes_received: u64,

    /// 浏览话题
    pub topics_entered: u64,

    /// 已读帖子
    pub posts_read_count: u64,

    /// 访问天数
    pub days_visited: u64,

    /// 创建帖子
    pub post_count: u64,

    /// 阅读时间 (sec)
    pub time_read: u64,

    /// 解决方案
    pub solved_count: u64,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Deserialize)]
#[serde(default)]
#[repr(transparent)]
/// <https://linux.do/u/{$username}.json>
pub(super) struct UserAll {
    /// 用户信息
    pub user: User,
}

#[derive(Debug, Clone, Default)]
#[derive(serde::Deserialize)]
#[serde(default)]
/// <https://linux.do/u/{$username}.json> -> user
pub(super) struct User {
    /// 用户 ID
    pub id: u64,

    /// 用户名
    pub username: Arc<str>,

    /// 用户昵称
    pub name: Option<Arc<str>>,

    /// 用户等级
    pub trust_level: u8,

    /// 用户 Bio (Raw)
    pub bio_raw: Option<Arc<str>>,

    /// 最后上线
    pub last_seen_at: DateTime<Utc>,

    /// 注册时间
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[derive(serde::Deserialize)]
/// General response for discourse API
pub(super) struct GeneralResponse<T = serde_json::Value> {
    #[serde(flatten)]
    data: T,

    #[serde(flatten)]
    error: Option<Box<ErrorDetails>>,
}

impl<T> GeneralResponse<T> {
    #[inline]
    /// Get result of the [`GeneralResponse`].
    pub(super) fn result(self) -> Result<T, Box<ErrorDetails>> {
        match self.error {
            None => Ok(self.data),
            Some(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(thiserror::Error)]
#[derive(serde::Deserialize)]
#[error("Discourse error: {error_type}, details {errors:?}")]
/// Error details.
///
/// Example:
///
/// ```no_run
/// {"errors":["The requested URL or resource could not be found."],"error_type":"not_found"}
/// ```
pub(super) struct ErrorDetails {
    /// errors
    pub errors: Vec<String>,

    /// error type
    pub error_type: String,
}
