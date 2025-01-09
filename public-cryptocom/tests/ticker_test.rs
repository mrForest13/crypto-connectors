use anyhow::Result;
use async_nats::Subscriber;
use futures::stream::Take;
use futures::StreamExt;
use prost::Message as ProstMessage;
use protocol::client::{NatsClient, NatsConfig};
use protocol::public::ticker::{TickerMessage, TickerRequest};
use protocol::public::types::Exchange;
use protocol::topics::{StreamTopic, Topic};
use public_cryptocom::client::ws_client::WsClient;
use public_cryptocom::config::ExchangeConfig;
use public_cryptocom::model::Market;
use public_cryptocom::ticker;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
use ws_mock::matchers::Any;
use ws_mock::ws_mock_server::{WsMock, WsMockServer};

const FIRST: &str = r#"{
  "id": 1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "instrument_name": "BTC_USD",
    "subscription": "ticker.BTC_USD",
    "channel": "ticker",
    "data": [
      {
        "h": "102780.56",
        "l": "96109.81",
        "a": "96441.70",
        "c": "-0.0526",
        "b": "96447.99",
        "bs": "1.68000",
        "k": "96448.00",
        "ks": "0.12219",
        "i": "BTC_USD",
        "v": "28786.2439",
        "vv": "2836123068.86",
        "oi": "0",
        "t": 1736286461888
      }
    ]
  }
}"#;

const SECOND: &str = r#"{
  "id": 1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "instrument_name": "BTC_USD",
    "subscription": "ticker.BTC_USD",
    "channel": "ticker",
    "data": [
      {
        "h": "102780.56",
        "l": "96109.81",
        "a": "96441.70",
        "c": "-0.0526",
        "b": "96449.99",
        "bs": "2.68000",
        "k": "96460.00",
        "ks": "0.33219",
        "i": "BTC_USD",
        "v": "28786.2439",
        "vv": "2836123068.86",
        "oi": "0",
        "t": 1736286461888
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
async fn stream_ticker_twice() -> Result<()> {
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
        ticker::stream::run(nats.clone(), ws_client.clone(), &exchange_config)
            .await
            .expect("running markets stream");
    });

    let market: Market = Market::new("btc".to_string(), "usd".to_string());
    let subject: StreamTopic = StreamTopic::ticker(Exchange::Cryptocom, &market);
    let request: TickerRequest = TickerRequest {};

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    nats_client
        .send_message(subject.snapshot(), request)
        .await?;

    let mut subscriber: Take<Subscriber> = nats_client.subscribe(subject).await?.take(2);

    if let Some(first) = subscriber.next().await {
        let response: TickerMessage = TickerMessage::decode(first.payload)?;

        assert_eq!(response.exchange, Exchange::Cryptocom as i32);
        assert_eq!(response.tick.is_some(), true);

        if let Some(tick) = response.tick {
            assert_eq!(tick.ask_price, "96448.00");
            assert_eq!(tick.bid_price, "96447.99");
        }
    }

    if let Some(second) = subscriber.next().await {
        let response: TickerMessage = TickerMessage::decode(second.payload)?;

        assert_eq!(response.exchange, Exchange::Cryptocom as i32);
        assert_eq!(response.tick.is_some(), true);

        if let Some(tick) = response.tick {
            assert_eq!(tick.ask_price, "96460.00");
            assert_eq!(tick.bid_price, "96449.99");
        }
    }

    Ok(())
}
