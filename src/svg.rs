pub(crate) mod moe_counter;

use std::{borrow::Cow, sync::LazyLock};

use chrono::{Datelike, Utc, Weekday};
use chrono_tz::Tz;
use macro_toolset::str_concat;

static AMMONIA: LazyLock<ammonia::Builder<'static>> = LazyLock::new(ammonia::Builder::default);

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
            r#"<g id="image"><line class="line" y1="20" y2="170" x1="300.5" x2="300.5"/>"#,
            r#"<image class="bg" href=""#,
            include_str!("../assets/image/marisa-kirisame.png.data"),
            r#"" transform="translate(300.5, 32) scale(0.5)"/></g>"#,
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

        str_concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 500 180" id="FmjApd" fr-init-rc="true">"#,
            // Static data
            SVG_STATIC_DATA,
            // Group: detail
            r#"<g id="detail">"#,
            r#"<text class="text" transform="translate(20 35)">欢迎您，第 "#,
            self.access_count,
            if self.access_count.is_some() {
                None
            } else {
                Some("NaN")
            },
            r#" 位访问本页面的朋友 🎉</text>"#,
            r#"<text class="text" transform="translate(20 65)">今天是 "#,
            now_month,
            r#" 月 "#,
            now_day,
            r#" 日，星期"#,
            now_weekday,
            r#"</text>"#,
            r#"<text class="text" transform="translate(20 95)">也是 "#,
            now_year,
            r#" 年的第 "#,
            ordinal,
            r#" 天</text>"#,
            r#"<text class="text" transform="translate(20 125)">距离 "#,
            now_year,
            r#" 年末还有 "#,
            ordinal_left,
            " 天</text>",
            self.note.map(|note| (
                r#"<text class="text" transform="translate(20 155)">"#,
                AMMONIA.clean(note),
                r#"</text>"#
            )),
            "</g>",
            // End SVG
            "</svg>"
        )
    }
}
