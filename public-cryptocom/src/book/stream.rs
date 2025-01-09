use crate::book::models::OrderBook;
use crate::book::state::OrderBookState;
use crate::client::response::WsResult;
use crate::client::ws_client::WsClient;
use crate::config::ExchangeConfig;
use crate::topics;
use crate::utils::handler::Event::Updated;
use crate::utils::handler::{Event, Handler};
use crate::utils::stream::handle_nats_subscription;
use anyhow::Result;
use connector::subscription::NatsSubscription;
use log::info;
use protocol::client::NatsClient;
use protocol::public::book::{OrderBookMessage, OrderBookRequest};
use protocol::topics::{SnapshotTopic, Topic};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;

const QUEUE: &str = "cryptocom.book";

pub async fn run(
    nats_client: Arc<NatsClient>,
    ws_client: Arc<WsClient>,
    config: &ExchangeConfig,
) -> Result<()> {
    let topic: SnapshotTopic = topics::order_book(&config.markets).snapshot();

    info!("Starting book stream processing");

    let nats_subscription: NatsSubscription<OrderBookRequest> =
        NatsSubscription::new(&nats_client, topic, QUEUE).await?;
    let shutdown: Receiver<()> = ws_client.subscribe_shutdown();
    let ws_subscription: Receiver<WsResult<OrderBook>> = ws_client.subscribe_book();
    let (message_handler, tickers): (Handler<OrderBook>, Sender<Event<OrderBook>>) =
        Handler::new(nats_client, ws_client, config);

    select! {
        result = message_handler.run::<OrderBookMessage, OrderBookState>(shutdown) => result,
        result = handle_nats_subscription(tickers.clone(), nats_subscription) => result,
        result = handle_ws_subscription(tickers.clone(), ws_subscription) => result
    }
}

async fn handle_ws_subscription(
    books: Sender<Event<OrderBook>>,
    mut subscription: Receiver<WsResult<OrderBook>>,
) -> Result<()> {
    while let Ok(result) = subscription.recv().await {
        if let Some(book) = result.data.first() {
            books.send(Updated(result.market, book.clone())).await?
        }
    }

    Ok(())
}
