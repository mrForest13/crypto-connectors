use crate::client::response::{ExchangeError, ExchangeResponse, HttpResult};
use crate::config::ExchangeConfig;
use crate::markets::models::Instrument;
use anyhow::{anyhow, Result};
use async_nats::Subject;
use chrono::Utc;
use connector::decoder::NatsEvent;
use connector::http_client::HttpClient;
use log::info;
use protocol::client::NatsClient;
use protocol::public::error::ErrorMessage;
use protocol::public::market::{Market, MarketsMessage, MarketsRequest};
use protocol::public::types::Exchange;
use reqwest::Url;
use std::sync::Arc;

type InstrumentsResponse = ExchangeResponse<HttpResult<Instrument>>;

pub struct RequestHandler {
    http_client: Arc<HttpClient>,
    nats_client: Arc<NatsClient>,
    markets_url: Url,
}

impl RequestHandler {
    pub fn new(
        http_client: Arc<HttpClient>,
        nats_client: Arc<NatsClient>,
        config: &ExchangeConfig,
    ) -> Result<Self> {
        Ok(RequestHandler {
            http_client,
            nats_client,
            markets_url: Url::parse(&config.markets_url)?,
        })
    }

    pub async fn process(&self, event: NatsEvent<MarketsRequest>) -> Result<()> {
        if let Some(reply) = event.reply {
            info!("Processing markets request");
            self.get_markets(event.message, reply).await
        } else {
            Err(anyhow!("No reply topic provided!"))
        }
    }

    async fn get_markets(&self, request: MarketsRequest, reply_topic: Subject) -> Result<()> {
        let response: Result<MarketsMessage, ErrorMessage> = self.call_api(request).await;

        match response {
            Ok(markets_message) => self
                .nats_client
                .send_message(reply_topic, markets_message)
                .await
                .map_err(|err| anyhow!(err)),
            Err(error_message) => self
                .nats_client
                .send_error(reply_topic, error_message)
                .await
                .map_err(|err| anyhow!(err)),
        }
    }

    async fn call_api(&self, request: MarketsRequest) -> Result<MarketsMessage, ErrorMessage> {
        self.http_client
            .get::<InstrumentsResponse, ExchangeError>(&self.markets_url)
            .await
            .map(|response| response.result.data)
            .map(|instruments| filter(instruments, request))
            .map(to_message)
    }
}

fn to_message(markets: Vec<Market>) -> MarketsMessage {
    MarketsMessage {
        timestamp: Utc::now().timestamp_millis(),
        exchange: Exchange::Cryptocom as i32,
        markets,
    }
}

fn filter(instruments: Vec<Instrument>, request: MarketsRequest) -> Vec<Market> {
    if request.symbols.is_empty() {
        instruments.iter().map(Market::from).collect()
    } else {
        instruments
            .iter()
            .map(Market::from)
            .filter(|market| request.symbols.contains(&market.symbol))
            .filter(|market| check(request.market_type, market.market_type))
            .collect()
    }
}

fn check(opt: Option<i32>, market_type: i32) -> bool {
    opt.map_or(true, |t| t == market_type)
}
