use std::{fs::File, net::SocketAddr, path::Path, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub(crate) struct Config {
    #[arg(short, long, default_value = "0.0.0.0:8989")]
    /// Listen address
    pub listen: SocketAddr,

    #[arg(long)]
    /// `access_key` for adding new counters
    ///
    /// If not set, no new counters can be added and use the existing ones from
    /// the config.
    pub access_key: Option<Arc<String>>,

    #[arg(short, long)]
    /// Authorized user ids
    pub user_id: Vec<Arc<str>>,

    #[arg(short, long, default_value_t = 131072)]
    /// Max number of counters
    ///
    /// Notice: for public service, this should not be set to `0`
    /// since your server may run out of memory.
    pub max_counter: usize,
}

impl Config {
    /// Parse command line arguments, or read from config file
    pub(crate) fn parse() -> Result<Self> {
        let args = Config::try_parse();

        let file = Path::new("./config.json");
        if file.exists() {
            tracing::info!("Reading config file from {}", file.to_str().unwrap());

            let fs = File::open(file).with_context(|| "Read config.json error")?;
            let config: Config =
                serde_json::from_reader(fs).with_context(|| "Parse config.json error")?;

            return Ok(config);
        }

        tracing::info!("Read command line arguments...");
        args.map_err(Into::into)
    }
}
