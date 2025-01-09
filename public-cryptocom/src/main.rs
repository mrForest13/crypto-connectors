use anyhow::{Context, Result};
use connector::http_client::HttpClient;
use connector::utils::check::nats_healthcheck;
use connector::utils::tracing;
use http::healthcheck::service::HealthcheckService;
use http::server::{base_router, HttpConfig};
use protocol::client::NatsClient;
use public_cryptocom::client::ws_client::WsClient;
use public_cryptocom::config::{load_config, AppConfig};
use public_cryptocom::{book, markets, ticker, trades};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::select;

#[tokio::main]
async fn main() -> Result<()> {
    tracing::init()?;

    let config: AppConfig = load_config()?;

    let http_client: Arc<HttpClient> = Arc::new(HttpClient::default());
    let nats_client: Arc<NatsClient> = Arc::new(NatsClient::new(&config.nats).await?);
    let ws_client: Arc<WsClient> = Arc::new(WsClient::new(&config.exchange)?);

    let healthcheck: HealthcheckService = nats_healthcheck(nats_client.clone());

    // maybe tokio spawn?
    let markets_stream_task =
        markets::stream::run(nats_client.clone(), http_client.clone(), &config.exchange);
    let ticker_stream_task =
        ticker::stream::run(nats_client.clone(), ws_client.clone(), &config.exchange);
    let trades_stream_task =
        trades::stream::run(nats_client.clone(), ws_client.clone(), &config.exchange);
    let books_stream_task =
        book::stream::run(nats_client.clone(), ws_client.clone(), &config.exchange);

    select! {
        ws = ws_client.run() => ws?,
        task = markets_stream_task => task?,
        task = ticker_stream_task => task?,
        task = trades_stream_task => task?,
        task = books_stream_task => task?,
        task = run_server(&config.http, healthcheck) => task?,
    }

    Ok(())
}

async fn run_server(config: &HttpConfig, service: HealthcheckService) -> Result<()> {
    let router = base_router(service);

    let listener: TcpListener = TcpListener::bind(config.address())
        .await
        .context("Error during server address binding")?;

    let server: () = axum::serve(listener, router)
        .await
        .context("Error during http server start")?;

    Ok(server)
}
