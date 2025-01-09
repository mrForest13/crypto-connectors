use anyhow::{Context, Result};
use std::env;
use std::str::FromStr;
use tracing::subscriber::set_global_default;
use tracing::Level;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::FmtSubscriber;

const ENV_LOG_LEVEL: &str = "RUST_LOG";
const DEFAULT_LOG_LEVEL: Level = Level::INFO;

pub fn init() -> Result<()> {
    let level: Level = env::var(ENV_LOG_LEVEL)
        .map(parse_level)
        .unwrap_or(DEFAULT_LOG_LEVEL);

    LogTracer::init()?;

    let subscriber: Subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    set_global_default(subscriber).context("Invalid setting for tracking")
}

fn parse_level(level: String) -> Level {
    Level::from_str(level.as_str()).unwrap_or(DEFAULT_LOG_LEVEL)
}
