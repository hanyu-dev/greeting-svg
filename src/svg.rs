pub(crate) mod linux_do_card;
pub(crate) mod moe_counter;

use std::{borrow::Cow, convert::Infallible, str::FromStr};

use chrono::{Datelike, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use macro_toolset::{str_concat, string::StringExtT};

use crate::utils::ammonia::get_filterd_note;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum BgType {
    /// Count down to Lunar New Year
    LunarNewYear,

    #[default]
    /// General background
    None,
}

impl FromStr for BgType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "lunar_new_year" => BgType::LunarNewYear,
            _ => BgType::None,
        })
    }
}

impl BgType {
    #[inline]
    fn svg_content(&self) -> impl StringExtT + use<> {
        match self {
            BgType::None => (
                r#"<g id="image"><line class="line" y1="20" y2="170" x1="300.5" x2="300.5"/><image class="bg" href=""#,
                include_str!("../assets/image/marisa-kirisame.png.data"),
                r#"" transform="translate(300.5, 32) scale(0.5)"/></g>"#,
            ),
            BgType::LunarNewYear => (
                r#"<g id="background"><image class="bg" href=""#,
                include_str!("../assets/image/new-year.jpeg.data"),
                r#"" transform="translate(0, 0) scale(0.25)"/></g>"#,
            ),
        }
    }
}

#[derive(Debug)]
/// Greeting data
pub(crate) struct GeneralImpl<'g> {
    /// Custom timezone
    pub tz: Tz,

    /// Access count
    pub access_count: Option<u64>,

    /// Count down type
    pub bg_type: BgType,

    /// Note
    pub note: Option<&'g Cow<'g, str>>,
}

impl GeneralImpl<'_> {
    /// Create a new [`GeneralImpl`]
    pub(crate) async fn generate(self) -> String {
        /// SVG static data
        static SVG_STATIC_DATA: &str = concat!(
            // CSS
            "<defs><style>",
            include_str!("../assets/theme/general/main.css"),
            "</style></defs>",
            // Title
            "<title>",
            "Cards | Jerry Zhou and Hantong Chen",
            "</title>",
        );

        let now = Utc::now().with_timezone(&self.tz);

        let now_year = now.year();
        let now_month = now.month();
        let now_day = now.day();

        let now_weekday = match now.weekday() {
            Weekday::Mon => "一",
            Weekday::Tue => "二",
            Weekday::Wed => "三",
            Weekday::Thu => "四",
            Weekday::Fri => "五",
            Weekday::Sat => "六",
            Weekday::Sun => "日",
        };

        let ordinal = now.ordinal();
        let ordinal_left = match self.bg_type {
            BgType::LunarNewYear => {
                const LUNAR_2025_NEW_YEAR_TIMESTAMP: i64 = 1738080000;

                chrono::Local
                    .timestamp_opt(LUNAR_2025_NEW_YEAR_TIMESTAMP, 0)
                    .unwrap()
                    .signed_duration_since(now)
                    .num_days()
            }
            BgType::None => {
                if (now_year % 4 == 0 && now_year % 100 != 0) || now_year % 400 == 0 {
                    366 - ordinal as i64
                } else {
                    365 - ordinal as i64
                }
            }
        };

        let note = match self.note {
            Some(note) => {
                let filtered_note = get_filterd_note(note, None, false).await;
                filtered_note.map_or((None, self.note), |note| (Some(note), None))
            }
            None => (None, None),
        };

        str_concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 500 140" fr-init-rc="true">"#,
            // Static data
            SVG_STATIC_DATA,
            self.bg_type.svg_content(),
            // Group: detail
            r#"<g id="detail">"#,
            r#"<text class="text" transform="translate(16 30)">欢迎您，"#,
            self.access_count
                .with_prefix("第 ")
                .with_suffix(" 位访问本页面的"),
            r#"朋友 🎉</text>"#,
            r#"<text class="text" transform="translate(16 60)">今天是新历 "#,
            now_year,
            r#" 年 "#,
            now_month,
            r#" 月 "#,
            now_day,
            r#" 日，星期"#,
            now_weekday,
            r#"</text>"#,
            r#"<text class="text" transform="translate(16 90)">已经是今年的第 "#,
            ordinal,
            r#" 天啦，离"#,
            match self.bg_type {
                BgType::LunarNewYear => "农历新年",
                BgType::None => "新历新年",
            },
            r#"还有 "#,
            ordinal_left,
            " 天</text>",
            note.with_prefix(r#"<text class="text" transform="translate(16 120)">"#)
                .with_suffix(r#"</text>"#),
            "</g>",
            // End SVG
            "</svg>"
        )
    }
}
