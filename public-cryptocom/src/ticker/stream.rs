use crate::client::response::WsResult;
use crate::client::ws_client::WsClient;
use crate::config::ExchangeConfig;
use crate::ticker::models::Ticker;
use crate::ticker::state::TickerState;
use crate::topics;
use crate::utils::handler::Event::Updated;
use crate::utils::handler::{Event, Handler};
use crate::utils::stream::handle_nats_subscription;
use anyhow::Result;
use connector::subscription::NatsSubscription;
use log::info;
use protocol::client::NatsClient;
use protocol::public::ticker::{TickerMessage, TickerRequest};
use protocol::topics::{SnapshotTopic, Topic};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

const QUEUE: &str = "cryptocom.ticker";

pub async fn run(
    nats_client: Arc<NatsClient>,
    ws_client: Arc<WsClient>,
    config: &ExchangeConfig,
) -> Result<()> {
    let topic: SnapshotTopic = topics::ticker(&config.markets).snapshot();

    info!("Starting ticker stream processing");

    let nats_subscription: NatsSubscription<TickerRequest> =
        NatsSubscription::new(&nats_client, topic, QUEUE).await?;
    let shutdown: Receiver<()> = ws_client.subscribe_shutdown();
    let ws_subscription: Receiver<WsResult<Ticker>> = ws_client.subscribe_ticker();
    let (message_handler, tickers): (Handler<Ticker>, Sender<Event<Ticker>>) =
        Handler::new(nats_client, ws_client, config);

    select! {
        result = message_handler.run::<TickerMessage, TickerState>(shutdown) => result,
        result = handle_nats_subscription(tickers.clone(), nats_subscription) => result,
        result = handle_ws_subscription(tickers.clone(), ws_subscription) => result
    }
}

async fn handle_ws_subscription(
    tickers: Sender<Event<Ticker>>,
    mut subscription: Receiver<WsResult<Ticker>>,
) -> Result<()> {
    while let Ok(result) = subscription.recv().await {
        if let Some(ticker) = result.data.first() {
            tickers.send(Updated(result.market, ticker.clone())).await?
        }
    }

    Ok(())
}
