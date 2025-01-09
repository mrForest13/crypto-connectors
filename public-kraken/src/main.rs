use crate::config::{load_config, AppConfig};
use anyhow::Context;
use connector::http_client::HttpClient;
use connector::utils::check::nats_healthcheck;
use connector::utils::tracing;
use http::healthcheck::service::HealthcheckService;
use http::server::{base_router, HttpConfig};
use protocol::client::NatsClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::select;

mod client;
mod config;
mod markets;
mod model;
mod topics;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::init()?;

    let config: AppConfig = load_config()?;

    let http_client: Arc<HttpClient> = Arc::new(HttpClient::default());
    let nats_client: Arc<NatsClient> = Arc::new(NatsClient::new(&config.nats).await?);

    let healthcheck: HealthcheckService = nats_healthcheck(nats_client.clone());

    let markets_stream_task =
        markets::stream::run(nats_client.clone(), http_client.clone(), &config.exchange);
    select! {
        task = markets_stream_task => task?,
        task = run_server(&config.http, healthcheck) => task?,
    }

    Ok(())
}

async fn run_server(config: &HttpConfig, service: HealthcheckService) -> anyhow::Result<()> {
    let router = base_router(service);

    let listener: TcpListener = TcpListener::bind(config.address())
        .await
        .context("Error during server address binding")?;

    let server: () = axum::serve(listener, router)
        .await
        .context("Error during http server start")?;

    Ok(server)
}
