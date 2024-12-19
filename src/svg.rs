pub(crate) mod moe_counter;

use std::borrow::Cow;

use chrono::{Datelike, Utc, Weekday};
use chrono_tz::Tz;
use macro_toolset::{str_concat, string::StringExtT};

use crate::utils::ammonia::get_filterd_note;

#[derive(Debug)]
/// Greeting data
pub(crate) struct GeneralImpl<'g> {
    /// Custom timezone
    pub tz: Tz,

    /// Access count
    pub access_count: Option<u64>,

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
            // Image on the right side
            r#"<g id="image"><line class="line" y1="20" y2="135" x1="300.5" x2="300.5"/>"#,
            r#"<image class="bg" href=""#,
            include_str!("../assets/image/marisa-kirisame.png.data"),
            r#"" transform="translate(300.5, 28) scale(0.42)"/></g>"#,
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
        let ordinal_left = if (now_year % 4 == 0 && now_year % 100 != 0) || now_year % 400 == 0 {
            366
        } else {
            365
        } - ordinal;

        let note = match self.note {
            Some(note) => {
                let filtered_note = get_filterd_note(note).await;
                filtered_note.map_or((None, self.note), |note| (Some(note), None))
            }
            None => (None, None),
        };

        str_concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 500 180" id="FmjApd" fr-init-rc="true">"#,
            // Static data
            SVG_STATIC_DATA,
            // Group: detail
            r#"<g id="detail">"#,
            r#"<text class="text" transform="translate(20 35)">欢迎您，"#,
            self.access_count
                .with_prefix("第 ")
                .with_suffix(" 位访问本页面的"),
            r#"朋友 🎉</text>"#,
            r#"<text class="text" transform="translate(20 65)">今天是 "#,
            now_year,
            r#" 年 "#,
            now_month,
            r#" 月 "#,
            now_day,
            r#" 日，星期"#,
            now_weekday,
            r#"</text>"#,
            r#"<text class="text" transform="translate(20 95)">已经是今年的第 "#,
            ordinal,
            r#" 天啦，离年末还有 "#,
            ordinal_left,
            " 天</text>",
            note.with_prefix(r#"<text class="text" transform="translate(20 125)">"#)
                .with_suffix(r#"</text>"#),
            "</g>",
            // End SVG
            "</svg>"
        )
    }
}
