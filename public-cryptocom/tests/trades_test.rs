use anyhow::Result;
use async_nats::Subscriber;
use futures::stream::Take;
use futures::StreamExt;
use prost::Message as ProstMessage;
use protocol::client::{NatsClient, NatsConfig};
use protocol::public::trade::{TradesMessage, TradesRequest};
use protocol::public::types::{Exchange, Side};
use protocol::topics::{StreamTopic, Topic};
use public_cryptocom::client::ws_client::WsClient;
use public_cryptocom::config::ExchangeConfig;
use public_cryptocom::model::Market;
use public_cryptocom::trades;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
use ws_mock::matchers::Any;
use ws_mock::ws_mock_server::{WsMock, WsMockServer};

const FIRST: &str = r#"{
  "id": -1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "instrument_name": "BTC_USD",
    "subscription": "trade.BTC_USD",
    "channel": "trade",
    "data": [
      {
        "d": "1736293040312188692",
        "t": 1736293040312,
        "p": "97092.61",
        "q": "0.01430",
        "s": "BUY",
        "i": "BTC_USD",
        "m": "4611686018585699621"
      }
    ]
  }
}"#;

const SECOND: &str = r#"{
  "id": -1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "instrument_name": "BTC_USD",
    "subscription": "trade.BTC_USD",
    "channel": "trade",
    "data": [
      {
        "d": "1736293040312188692",
        "t": 1736293040312,
        "p": "99092.61",
        "q": "1.01430",
        "s": "SELL",
        "i": "BTC_USD",
        "m": "4611686018585699622"
      }
    ]
  }
}"#;

fn nats_conf() -> NatsConfig {
    NatsConfig {
        host: "0.0.0.0".to_string(),
        port: 4222,
        max_reconnects: 0,
    }
}

fn exchange_conf(uri: String) -> ExchangeConfig {
    ExchangeConfig {
        ws_url: format!("{}/ws", uri),
        markets_url: format!("{}/markets", uri),
        markets: Market::new("*".to_string(), "*".to_string()),
        max_concurrency: 2,
        max_buffer_size: 10,
    }
}

#[tokio::test]
async fn stream_trades_twice_times() -> Result<()> {
    let server: WsMockServer = WsMockServer::start().await;

    WsMock::new()
        .matcher(Any::new())
        .respond_with(Message::Text(String::from(FIRST)))
        .respond_with(Message::Text(String::from(SECOND)))
        .mount(&server)
        .await;

    let nats_config: NatsConfig = nats_conf();
    let exchange_config: ExchangeConfig = exchange_conf(server.uri().await);

    let nats_client: Arc<NatsClient> = Arc::new(NatsClient::new(&nats_config).await?);
    let ws_client: Arc<WsClient> = Arc::new(WsClient::new(&exchange_config)?);

    let ws: Arc<WsClient> = ws_client.clone();
    tokio::task::spawn(async move {
        ws.run().await.expect("running ws stream");
    });

    let nats: Arc<NatsClient> = nats_client.clone();
    tokio::task::spawn(async move {
        trades::stream::run(nats.clone(), ws_client.clone(), &exchange_config)
            .await
            .expect("running markets stream");
    });

    let market: Market = Market::new("btc".to_string(), "usd".to_string());
    let subject: StreamTopic = StreamTopic::trades(Exchange::Cryptocom, &market);
    let request: TradesRequest = TradesRequest {};

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    nats_client
        .send_message(subject.snapshot(), request)
        .await?;

    let mut subscriber: Take<Subscriber> = nats_client.subscribe(subject).await?.take(2);

    if let Some(first) = subscriber.next().await {
        let response: TradesMessage = TradesMessage::decode(first.payload)?;

        assert_eq!(response.exchange, Exchange::Cryptocom as i32);
        assert_eq!(response.trades.len(), 1);

        if let Some(trade) = response.trades.first() {
            assert_eq!(trade.size, "96448.00");
            assert_eq!(trade.rate, "0.01430");
            assert_eq!(trade.side, Side::Buy as i32);
        }
    }

    if let Some(second) = subscriber.next().await {
        let response: TradesMessage = TradesMessage::decode(second.payload)?;

        assert_eq!(response.exchange, Exchange::Cryptocom as i32);
        assert_eq!(response.trades.len(), 1);

        if let Some(trade) = response.trades.first() {
            assert_eq!(trade.size, "99092.61");
            assert_eq!(trade.rate, "1.01430");
            assert_eq!(trade.side, Side::Sell as i32);
        }
    }

    Ok(())
}
