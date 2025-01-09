use anyhow::Result;
use connector::http_client::HttpClient;
use mockito::{Server, ServerGuard};
use prost::Message;
use protocol::client::{NatsClient, NatsConfig};
use protocol::public::market::{MarketType, MarketsMessage, MarketsRequest};
use protocol::public::types::Exchange;
use protocol::topics::RequestTopic;
use public_cryptocom::config::ExchangeConfig;
use public_cryptocom::markets;
use public_cryptocom::model::Market;
use std::sync::Arc;

const OK_BODY: &str = r#"{
  "id": -1,
  "method": "public/get-instruments",
  "code": 0,
  "result": {
    "data": [
      {
        "symbol": "BTC_USD",
        "inst_type": "CCY_PAIR",
        "display_name": "BTC/USD",
        "base_ccy": "BTC",
        "quote_ccy": "USD",
        "quote_decimals": 2,
        "quantity_decimals": 5,
        "price_tick_size": "0.01",
        "qty_tick_size": "0.00001",
        "max_leverage": "50",
        "tradable": true,
        "expiry_timestamp_ms": 0,
        "beta_product": false,
        "margin_buy_enabled": true,
        "margin_sell_enabled": true
      },
      {
        "symbol": "ETH_USD",
        "inst_type": "CCY_PAIR",
        "display_name": "ETH/USD",
        "base_ccy": "ETH",
        "quote_ccy": "USD",
        "quote_decimals": 2,
        "quantity_decimals": 4,
        "price_tick_size": "0.01",
        "qty_tick_size": "0.0001",
        "max_leverage": "50",
        "tradable": true,
        "expiry_timestamp_ms": 0,
        "beta_product": false,
        "margin_buy_enabled": true,
        "margin_sell_enabled": true
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

fn exchange_conf(server: &ServerGuard) -> ExchangeConfig {
    ExchangeConfig {
        ws_url: format!("{}/ws", server.url()),
        markets_url: format!("{}/markets", server.url()),
        markets: Market::new("*".to_string(), "*".to_string()),
        max_concurrency: 2,
        max_buffer_size: 10,
    }
}

#[tokio::test]
async fn return_all_markets() -> Result<()> {
    let mut server: ServerGuard = Server::new_async().await;
    let _ = server.mock("GET", "/markets").with_body(OK_BODY).create();

    let nats_config: NatsConfig = nats_conf();
    let exchange_config: ExchangeConfig = exchange_conf(&server);

    let http_client: Arc<HttpClient> = Arc::new(HttpClient::default());
    let nats_client: Arc<NatsClient> = Arc::new(NatsClient::new(&nats_config).await?);

    let nats: Arc<NatsClient> = nats_client.clone();
    tokio::task::spawn(async move {
        markets::stream::run(nats.clone(), http_client.clone(), &exchange_config)
            .await
            .expect("running markets stream");
    });

    let subject: RequestTopic = RequestTopic::markets(Exchange::Cryptocom);
    let request: MarketsRequest = MarketsRequest {
        symbols: vec![],
        market_type: Some(MarketType::Spot as i32),
    };

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let message = nats_client.send_request(subject, request).await?;
    let response = MarketsMessage::decode(message.payload)?;

    assert_eq!(response.exchange, Exchange::Cryptocom as i32);
    assert_eq!(response.markets.len(), 2);
    assert_eq!(response.markets[0].symbol, "btc_usd");
    assert_eq!(response.markets[1].symbol, "eth_usd");

    Ok(())
}
