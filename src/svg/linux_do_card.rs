//! Linux.do CARD

mod cache;
mod model;
mod upstream;

use std::sync::{Arc, LazyLock};

use macro_toolset::{
    str_concat,
    string::{NumStr, StringExtT},
};

use crate::utils::ammonia::get_filterd_note;

#[derive(Debug, Clone, Default)]
pub(crate) struct LinuxDoCardImpl<'i, V = &'i str> {
    user: Option<&'i str>,
    custom_bio: Option<V>,
    filtered_bio: Option<Arc<str>>,
}

impl<'i> LinuxDoCardImpl<'i> {
    pub(crate) const fn new(user: Option<&'i str>) -> Self {
        Self {
            user,
            custom_bio: None,
            filtered_bio: None,
        }
    }

    const fn empty() -> Self {
        Self {
            user: None,
            custom_bio: None,
            filtered_bio: None,
        }
    }
}

impl<'i, V> LinuxDoCardImpl<'i, V>
where
    V: AsRef<str>,
{
    pub(crate) async fn generate(self, _count: u64) -> String {
        static DEFAULT_EMPTY_USER_INFO: LazyLock<String> =
            LazyLock::new(|| LinuxDoCardImpl::empty().create(&model::UserInfo::default()));

        if let Some(user) = self.user {
            if let Some(v) = get_or_fetch(user).await {
                return self.create(&v);
            }
        }

        DEFAULT_EMPTY_USER_INFO.clone()
    }

    pub(crate) async fn set_custom_bio<NV>(self, custom_bio: Option<NV>) -> LinuxDoCardImpl<'i, NV>
    where
        NV: AsRef<str>,
    {
        let filtered_bio = if let Some(custom_bio) = custom_bio.as_ref() {
            get_filterd_note(custom_bio.as_ref(), None, false).await
        } else {
            None
        };

        LinuxDoCardImpl {
            user: self.user,
            custom_bio,
            filtered_bio,
        }
    }

    fn create(self, user_info: &model::UserInfo) -> String {
        str_concat!(
            r#"
            <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 600 270" fr-init-rc="true">
                <defs>
                    <style>
                        svg { background-color: rgba(0, 0, 0, 0); }
                        #info .text { font-size: 16px; fill: rgba(0, 140, 255, 1); font-weight: lighter; }
                        #summary .text { font-size: 16px; fill: rgba(0, 140, 255, 1); font-weight: lighter; }
                        #edit .text { font-size: 6px; fill: rgba(0, 140, 255, 1); font-weight: lighter; }
                    </style>
                </defs>
                <g id="info">
                    <text class="text" transform="translate(30 50)">"#,
            &user_info.user.username,
            r#" ("#,
            match user_info.user.trust_level {
                0 => "游客",
                1 => "🚲一级新萌",
                2 => "🚗二级老萌",
                3 => "🚅三级大佬",
                4 => "🚀站长本佬",
                _ => "✨突破天际",
            },
            r#")</text>
                    <text class="text" transform="translate(30 80)">"#,
            self.filtered_bio
                .as_ref()
                .map(AsRef::as_ref)
                .or(self.custom_bio.as_ref().map(AsRef::as_ref))
                .unwrap_or_else(|| &user_info.user.bio_excerpt), // BIO
            r#"</text>
                    <text class="text" transform="translate(30 110)">🕒注册时间</text>
                    <text class="text" transform="translate(330 110)">🕗最近上线</text>
                    <text class="text" transform="translate(150 110)">"#,
            cal_time_delta(user_info.user.created_at),
            r#"</text>
                    <text class="text" transform="translate(450 110)">"#,
            cal_time_delta(user_info.user.last_seen_at),
            r#"</text>
                </g>
                <line x1="30" y1="120" x2="570" y2="120" stroke="rgba(211, 211, 211, 1)" stroke-width="1"/>
                <g id="summary">
                    <text class="text" transform="translate(30 150)">🛎️访问天数</text>
                    <text class="text" transform="translate(30 180)">⌛阅读时间</text>
                    <text class="text" transform="translate(30 210)">📰浏览话题</text>
                    <text class="text" transform="translate(30 240)">📑已读帖子</text>
                    <text class="text" transform="translate(330 150)">💝已送出赞</text>
                    <text class="text" transform="translate(330 180)">👍已收到赞</text>
                    <text class="text" transform="translate(330 210)">📖创建帖子</text>
                    <text class="text" transform="translate(330 240)">💡解决方案</text>
                    <text class="text" transform="translate(150 150)">"#,
            user_info.user_summary.days_visited, // 访问天数
            r#"</text>
                    <text class="text" transform="translate(150 180)">"#,
            duration_human_format(user_info.user_summary.time_read), // 阅读时间
            r#"</text>
                    <text class="text" transform="translate(150 210)">"#,
            user_info.user_summary.topics_entered, // 浏览话题
            r#"</text>
                    <text class="text" transform="translate(150 240)">"#,
            user_info.user_summary.posts_read_count, // 已读帖子
            r#"</text>
                    <text class="text" transform="translate(450 150)">"#,
            user_info.user_summary.likes_given, // 已送出赞
            r#"</text>
                    <text class="text" transform="translate(450 180)">"#,
            user_info.user_summary.likes_received, // 已收到赞
            r#"</text>
                    <text class="text" transform="translate(450 210)">"#,
            user_info.user_summary.post_count, // 创建帖子
            r#"</text>
                    <text class="text" transform="translate(450 240)">"#,
            user_info.user_summary.solved_count, // 解决方案
            r#"</text>
                </g>
                <line x1="30" y1="255" x2="570" y2="255" stroke="rgba(211, 211, 211, 1)" stroke-width="1"/>
                <g id="edit">
                <text class="text" transform="translate(30 270)">Greeting SVG (MIT License), modified from `linuxdo-card` created by zjkal</text>
                <text class="text" transform="translate(330 270)">Updated: "#,
            user_info
                .created
                .map(|instant| {
                    (
                        Some(
                            (chrono::Local::now() - instant.elapsed())
                                .to_utc()
                                .to_rfc3339(),
                        ),
                        None,
                    )
                })
                .unwrap_or_else(|| (None, Some("... [FETCHING UPSTREAM IN BACKGROUND]"))),
            r#"</text>
                </g>
            </svg>
            "#
        )
    }
}

#[tracing::instrument(level = "debug")]
async fn get_or_fetch(user: &str) -> Option<Arc<model::UserInfo>> {
    match cache::get_cache(user) {
        Some((cache, false)) => {
            tracing::debug!("Cache hit");
            Some(cache)
        }
        Some((cache, true)) => {
            tracing::debug!("Cache expired, try to refresh");

            let key: Arc<str> = user.into();
            tokio::spawn(async move {
                match upstream::fetch(&key).await {
                    Ok(value) => cache::write_cache(key, value).await,
                    Err(e) => {
                        tracing::error!("Fetch upstream data error: {e:#?}");
                    }
                }
            });

            Some(cache)
        }
        None => {
            tracing::debug!("Cache missed, try fetch in background");

            let user = Arc::from(user);
            tokio::spawn(async move {
                match upstream::fetch(&user).await {
                    Ok(value) => cache::write_cache(user, value).await,
                    Err(e) => {
                        tracing::error!("Fetch upstream data error: {e:#?}");
                    }
                }
            });

            None
        }
    }
}

const SPEC_MINUTE_SECS: u64 = 60;
const SPEC_HOUR_SECS: u64 = 3600;
const SPEC_DAY_SECS: u64 = 86400;
const SPEC_WEEK_SECS: u64 = SPEC_DAY_SECS * 7;
const SPEC_MONTH_SECS: u64 = SPEC_DAY_SECS * 30;
const SPEC_YEAR_SECS: u64 = SPEC_DAY_SECS * 365;

fn duration_human_format(duration: u64) -> impl StringExtT {
    match duration {
        0..SPEC_HOUR_SECS => (
            NumStr::new_default(duration as f64 / SPEC_MINUTE_SECS as f64).set_resize_len::<2>(),
            " 分钟",
        ),
        SPEC_HOUR_SECS.. => (
            NumStr::new_default(duration as f64 / SPEC_HOUR_SECS as f64).set_resize_len::<2>(),
            " 小时",
        ),
    }
}

fn cal_time_delta<Tz: chrono::TimeZone>(time: chrono::DateTime<Tz>) -> Option<impl StringExtT> {
    let created_delta = chrono::Local::now().signed_duration_since(time);
    let created_delta_sec = created_delta.num_seconds();

    if created_delta_sec.is_negative() {
        tracing::error!("Invalid time setting, check the local clock!");
        None
    } else {
        Some(match created_delta_sec as u64 {
            0..SPEC_HOUR_SECS => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_MINUTE_SECS as f64)
                    .set_integer_only::<true>(),
                " 分钟前",
            ),
            SPEC_HOUR_SECS..SPEC_DAY_SECS => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_HOUR_SECS as f64)
                    .set_integer_only::<true>(),
                " 个小时前",
            ),
            SPEC_DAY_SECS..SPEC_WEEK_SECS => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_DAY_SECS as f64)
                    .set_integer_only::<true>(),
                " 天前",
            ),
            SPEC_WEEK_SECS..SPEC_MONTH_SECS => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_WEEK_SECS as f64)
                    .set_integer_only::<true>(),
                " 周前",
            ),
            SPEC_MONTH_SECS..SPEC_YEAR_SECS => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_MONTH_SECS as f64)
                    .set_integer_only::<true>(),
                " 个月前",
            ),
            SPEC_YEAR_SECS.. => (
                NumStr::new_default(created_delta_sec as f64 / SPEC_YEAR_SECS as f64)
                    .set_integer_only::<true>(),
                " 年前",
            ),
        })
    }
}

// #[test]
// fn t() {
//     macro_toolset::init_tracing_simple!();

//     let time: chrono::DateTime<chrono::Utc> =
// "2024-10-08T15:40:37.485Z".parse().unwrap();     let data =
// cal_time_delta(time).unwrap();     data.to_string_ext();

//     duration_human_format(376973).to_string_ext();
// }

// #[tokio::test]
// async fn test() {
//     macro_toolset::init_tracing_simple!();

//     let data = &*get_or_fetch("hantong").await.unwrap();
//     data;
//     let data = &*get_or_fetch("hantong").await.unwrap();
//     data;

//     tokio::time::sleep(std::time::Duration::from_secs(6)).await;
//     let data = &*get_or_fetch("hantong").await.unwrap();
//     data;
// }
