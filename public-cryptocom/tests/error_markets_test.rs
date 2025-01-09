use anyhow::Result;
use connector::http_client::HttpClient;
use mockito::{Server, ServerGuard};
use prost::Message;
use protocol::client::{NatsClient, NatsConfig};
use protocol::public::error::{ErrorCode, ErrorMessage};
use protocol::public::market::{MarketType, MarketsRequest};
use protocol::public::types::Exchange;
use protocol::topics::RequestTopic;
use public_cryptocom::config::ExchangeConfig;
use public_cryptocom::markets;
use public_cryptocom::model::Market;
use std::sync::Arc;

const ERROR_BODY: &str = r#"{
  "code" : 40004,
  "message" : "Invalid body"
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
async fn return_error_message() -> Result<()> {
    let mut server: ServerGuard = Server::new_async().await;
    let _ = server
        .mock("GET", "/markets")
        .with_body(ERROR_BODY)
        .create();

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
        symbols: vec!["eth_usd".to_string()],
        market_type: Some(MarketType::Spot as i32),
    };

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let message = nats_client.send_request(subject, request).await?;
    let response = ErrorMessage::decode(message.payload)?;

    assert_eq!(response.code, ErrorCode::UnknownCode as i32);
    assert_eq!(response.message, "Error during request!");
    assert_eq!(response.exchange_message, None);

    Ok(())
}
