use crate::markets_example::Market;
use connectors_sdk::connector::PublicConnector;
use connectors_sdk::subscription::NatsStream;
use futures::stream::Take;
use futures::StreamExt;
use protocol::client::{NatsClient, NatsConfig};
use protocol::public::book::OrderBookMessage;
use protocol::public::types::Exchange;

mod markets_example;

#[tokio::main]
async fn main() {
    let exchange: Exchange = Exchange::Cryptocom;
    let config: NatsConfig = NatsConfig {
        host: "0.0.0.0".to_string(),
        port: 4222,
        max_reconnects: 0,
    };

    let client: NatsClient = NatsClient::new(&config).await.expect("Nats error");
    let connector: PublicConnector = PublicConnector::new(client);

    let market = Market {
        from: "btc".into(),
        to: "usd".into(),
    };

    let mut subscription: Take<NatsStream<OrderBookMessage>> = connector
        .order_book(exchange, market)
        .await
        .expect("Failed to get book stream")
        .take(3);

    while let Some(message) = subscription.next().await {
        println!("{:?}", message);
    }

    println!("disconnected");
}
