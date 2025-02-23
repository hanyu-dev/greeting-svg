//! Linux.do CARD

mod cache;
mod model;
mod upstream;

use std::sync::{Arc, LazyLock};

use chrono_tz::Tz;
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
    tz: Tz,
    from_linux_do: bool,
}

impl<'i> LinuxDoCardImpl<'i> {
    pub(crate) const fn new(user: &'i str, tz: Tz, from_linux_do: bool) -> Self {
        Self {
            user: Some(user),
            custom_bio: None,
            filtered_bio: None,
            tz,
            from_linux_do,
        }
    }

    const fn empty() -> Self {
        Self {
            user: None,
            custom_bio: None,
            filtered_bio: None,
            tz: Tz::Asia__Shanghai,
            from_linux_do: false,
        }
    }
}

impl<'i, V> LinuxDoCardImpl<'i, V>
where
    V: AsRef<str>,
{
    pub(crate) async fn generate(self, count: Option<u64>) -> String {
        static DEFAULT_EMPTY_USER_INFO: LazyLock<String> =
            LazyLock::new(|| LinuxDoCardImpl::empty().create(&model::UserInfo::default()));

        cache::try_init_cache_update_queue().await;

        if let Some(user) = self.user {
            if let Some(v) = get_or_fetch(user, self.from_linux_do || count.is_some()).await {
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
            tz: self.tz,
            from_linux_do: self.from_linux_do,
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
                        #edit .text { font-size: 10px; fill: rgba(0, 140, 255, 1); font-weight: lighter; }
                    </style>
                </defs>
                <g id="info">
                    <text class="text" transform="translate(30 30)">"#,
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
                    <text class="text" transform="translate(30 60)">"#,
            self.filtered_bio
                .as_ref()
                .map(AsRef::as_ref)
                .or(self.custom_bio.as_ref().map(AsRef::as_ref))
                .or(user_info.user.bio_raw.as_ref().map(AsRef::as_ref))
                .unwrap_or("小白一枚"), // BIO
            r#"</text>
                    <text class="text" transform="translate(30 90)">🕒注册时间</text>
                    <text class="text" transform="translate(330 90)">🕗最近上线</text>
                    <text class="text" transform="translate(150 90)">"#,
            cal_time_delta(user_info.user.created_at),
            r#"</text>
                    <text class="text" transform="translate(450 90)">"#,
            cal_time_delta(user_info.user.last_seen_at),
            r#"</text>
                </g>
                <line x1="30" y1="100" x2="570" y2="100" stroke="rgba(211, 211, 211, 1)" stroke-width="1"/>
                <g id="summary">
                    <text class="text" transform="translate(30 130)">🛎️访问天数</text>
                    <text class="text" transform="translate(30 160)">⌛阅读时间</text>
                    <text class="text" transform="translate(30 190)">📰浏览话题</text>
                    <text class="text" transform="translate(30 220)">📑已读帖子</text>
                    <text class="text" transform="translate(330 130)">💝已送出赞</text>
                    <text class="text" transform="translate(330 160)">👍已收到赞</text>
                    <text class="text" transform="translate(330 190)">📖创建帖子</text>
                    <text class="text" transform="translate(330 220)">💡解决方案</text>
                    <text class="text" transform="translate(150 130)">"#,
            user_info.user_summary.days_visited, // 访问天数
            r#"</text>
                    <text class="text" transform="translate(150 160)">"#,
            duration_human_format(user_info.user_summary.time_read), // 阅读时间
            r#"</text>
                    <text class="text" transform="translate(150 190)">"#,
            user_info.user_summary.topics_entered, // 浏览话题
            r#"</text>
                    <text class="text" transform="translate(150 220)">"#,
            user_info.user_summary.posts_read_count, // 已读帖子
            r#"</text>
                    <text class="text" transform="translate(450 130)">"#,
            user_info.user_summary.likes_given, // 已送出赞
            r#"</text>
                    <text class="text" transform="translate(450 160)">"#,
            user_info.user_summary.likes_received, // 已收到赞
            r#"</text>
                    <text class="text" transform="translate(450 190)">"#,
            user_info.user_summary.post_count, // 创建帖子
            r#"</text>
                    <text class="text" transform="translate(450 220)">"#,
            user_info.user_summary.solved_count, // 解决方案
            r#"</text>
                </g>
                <line x1="30" y1="235" x2="570" y2="235" stroke="rgba(211, 211, 211, 1)" stroke-width="1"/>
                <g id="edit">
                <text class="text" transform="translate(30 250)">Greeting SVG (originated from `linuxdo-card` by zjkal)</text>
                <text class="text" transform="translate(330 250)">Updated: "#,
            user_info
                .created
                .map(|instant| {
                    (
                        Some(
                            (chrono::Local::now() - instant.elapsed())
                                .with_timezone(&self.tz)
                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
                        ),
                        None,
                    )
                })
                .unwrap_or_else(|| (None, Some("... [FETCHING UPSTREAM]"))),
            r#"</text>
                </g>
            </svg>
            "#
        )
    }
}

#[tracing::instrument(level = "debug")]
async fn get_or_fetch(user: &str, authorized: bool) -> Option<Arc<model::UserInfo>> {
    let (cached, need_fetch) = cache::get_cache_or_fetch(user, authorized);

    if let Some(need_fetch) = need_fetch {
        need_fetch.await;
    }

    cached
}

const SPEC_MINUTE_SECS: u64 = 60;
const SPEC_HOUR_SECS: u64 = SPEC_MINUTE_SECS * 60;
const SPEC_DAY_SECS: u64 = SPEC_HOUR_SECS * 24;
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
