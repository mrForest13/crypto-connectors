use anyhow::{Context, Result};
use config::builder::DefaultState;
use config::{Config, ConfigBuilder, Environment, File, FileFormat, FileSourceFile};
use serde::de::DeserializeOwned;
use tracing::info;

pub fn load_file<T: DeserializeOwned>(path: &str, name: &str) -> Result<T> {
    let path: String = format!("{}/{}.toml", path, name);

    info!("Loading config from {}", path);

    let file: File<FileSourceFile, FileFormat> = config::File::with_name(&path);

    let prefix: String = name.to_uppercase();
    let builder: ConfigBuilder<DefaultState> = Config::builder()
        .add_source(file)
        .add_source(Environment::with_prefix(&prefix));

    let config: T = builder
        .build()?
        .try_deserialize::<T>()
        .with_context(|| format!("Cannot load config for {}", name))?;

    Ok(config)
}
