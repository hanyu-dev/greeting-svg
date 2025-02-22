//! Linux.do cards, upstream API

use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use macro_toolset::str_concat;
use parking_lot::Mutex;
use reqwest::Client;

use super::model;
use crate::utils::{GENERAL_USER_AGENT, ammonia::get_filterd_note};

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .danger_accept_invalid_certs(cfg!(debug_assertions))
        .danger_accept_invalid_hostnames(cfg!(debug_assertions))
        // .proxy(reqwest::Proxy::all("http://127.0.0.1:8888").unwrap())
        .timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(120))
        .tcp_keepalive(Duration::from_secs(120))
        .http2_keep_alive_while_idle(true)
        .http2_keep_alive_interval(Duration::from_secs(15))
        .http2_keep_alive_timeout(Duration::from_secs(12))
        .user_agent(GENERAL_USER_AGENT)
        .build()
        .expect("will not fail here")
});

// 土法速率限制
async fn wait(secs: u64) {
    static GLOBAL_LIMITTER_LAST_ACCESS: LazyLock<Mutex<Option<Instant>>> =
        LazyLock::new(|| Mutex::new(Some(Instant::now())));

    // Simple rate limitter, burst=1req/{secs}...
    loop {
        // Make compiler happy
        let elasped = GLOBAL_LIMITTER_LAST_ACCESS.lock().map(|v| v.elapsed());
        if let Some(elasped) = elasped {
            if let Some(wait) = Duration::from_secs(secs).checked_sub(elasped) {
                tracing::trace!("Initiative request rate limitation exceeded, wait {wait:?}");
                tokio::time::sleep(wait).await;
                continue;
            }
        }

        *GLOBAL_LIMITTER_LAST_ACCESS.lock() = Some(Instant::now());
        break;
    }
}

#[tracing::instrument(level = "debug", ret)]
pub(super) async fn fetch(user_name: &str) -> Result<model::UserInfo> {
    if user_name.is_empty() {
        bail!("Empty user name!")
    }

    wait(1).await;
    let mut user = CLIENT
        .get(str_concat!("https://linux.do/u/", user_name, ".json"))
        .send()
        .await
        .context("Fetch response from https://linux.do/u/{$username}.json error")?
        .error_for_status()
        .context("Fetch response from https://linux.do/u/{$username}.json status error")?
        .json::<model::GeneralResponse<model::UserAll>>()
        .await
        .context("Parse json response from https://linux.do/u/{$username}.json error")?
        .result()?
        .user;

    // 32 中文 (1中文字符 = 2英文字符), 72 英文
    // abcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefga

    //  Filter
    if let Some(bio_raw) = &user.bio_raw {
        if let Some(bio_raw) = get_filterd_note(bio_raw, None, false).await {
            user.bio_raw = Some(bio_raw);
        }
    }

    wait(1).await;
    let user_summary = CLIENT
        .get(str_concat!(
            "https://linux.do/u/",
            user_name,
            "/summary.json"
        ))
        .send()
        .await
        .context("Fetch response from https://linux.do/u/{$username}/summary.json error")?
        .error_for_status()
        .context("Fetch response from https://linux.do/u/{$username}.json status error")?
        .json::<model::GeneralResponse<model::UserSummaryAll>>()
        .await
        .context("Parse json response from https://linux.do/u/{$username}/summary.json error")?
        .result()?
        .user_summary;

    Ok(model::UserInfo {
        created: Some(Instant::now()),
        user,
        user_summary,
    })
}

// #[tokio::test]
// async fn test() {
//     macro_toolset::init_tracing_simple!();

//     let user = "hantong";
//     // let data = CLIENT
//     //     .get(str_concat!("https://linux.do/u/", user, "/summary.json"))
//     //     .send()
//     //     .await
//     //     .unwrap()
//     //     .text()
//     //     .await
//     //     .unwrap();

//     let mut joinset = tokio::task::JoinSet::new();

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 1");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 2");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 3");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 4");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 5");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 6");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 7");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 8");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 9");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 10");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 11");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 12");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 13");
//     });

//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 14");
//     });
//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 15");
//     });
//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 16");
//     });
//     joinset.spawn(async {
//         fetch(user).await;
//         println!("OK from thread 17");
//     });

//     joinset.join_all().await;
// }
