use crate::client::response::WsResult;
use crate::client::ws_client::WsClient;
use crate::config::ExchangeConfig;
use crate::topics;
use crate::trades::models::Transaction;
use crate::trades::state::TradesState;
use crate::utils::handler::Event::Updated;
use crate::utils::handler::{Event, Handler};
use crate::utils::stream::handle_nats_subscription;
use anyhow::Result;
use connector::subscription::NatsSubscription;
use log::info;
use protocol::client::NatsClient;
use protocol::public::trade::{TradesMessage, TradesRequest};
use protocol::topics::{SnapshotTopic, Topic};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

const QUEUE: &str = "cryptocom.trades";

pub async fn run(
    nats_client: Arc<NatsClient>,
    ws_client: Arc<WsClient>,
    config: &ExchangeConfig,
) -> Result<()> {
    let topic: SnapshotTopic = topics::trades(&config.markets).snapshot();

    info!("Starting trades stream processing");

    let nats_subscription: NatsSubscription<TradesRequest> =
        NatsSubscription::new(&nats_client, topic, QUEUE).await?;
    let shutdown: Receiver<()> = ws_client.subscribe_shutdown();
    let ws_subscription: Receiver<WsResult<Transaction>> = ws_client.subscribe_trade();
    let (message_handler, trades): (Handler<Vec<Transaction>>, Sender<Event<Vec<Transaction>>>) =
        Handler::new(nats_client, ws_client, config);

    select! {
        result = message_handler.run::<TradesMessage, TradesState>(shutdown) => result,
        result = handle_nats_subscription(trades.clone(), nats_subscription) => result,
        result = handle_ws_subscription(trades.clone(), ws_subscription) => result
    }
}

async fn handle_ws_subscription(
    tickers: Sender<Event<Vec<Transaction>>>,
    mut subscription: Receiver<WsResult<Transaction>>,
) -> Result<()> {
    while let Ok(result) = subscription.recv().await {
        if !result.data.is_empty() {
            tickers
                .send(Updated(result.market.clone(), result.data))
                .await?
        }
    }

    Ok(())
}
