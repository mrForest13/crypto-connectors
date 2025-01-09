use crate::model::Market;
use anyhow::Result;
use connector::config::load_file;
use http::server;
use log::info;
use protocol::client;
use serde::Deserialize;
use std::env;

const ENV_PATH: &str = "CONFIGURATION_PATH";
const DEFAULT_PATH: &str = "public-cryptocom/resources";

#[derive(Deserialize)]
pub struct ExchangeConfig {
    pub ws_url: String,
    pub markets_url: String,
    pub markets: Market,
    pub max_concurrency: usize,
    pub max_buffer_size: usize,
}

pub struct AppConfig {
    pub http: server::HttpConfig,
    pub nats: client::NatsConfig,
    pub exchange: ExchangeConfig,
}

pub fn load_config() -> Result<AppConfig> {
    let path: String = env::var(ENV_PATH).unwrap_or(DEFAULT_PATH.to_string());

    let http = load_file(&path, "http")?;
    let nats = load_file(&path, "nats")?;
    let exchange = load_file(&path, "exchange")?;

    info!("Application config loaded successfully!");

    Ok(AppConfig {
        http,
        nats,
        exchange,
    })
}
