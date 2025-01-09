use crate::model::Market;
use chrono::Utc;
use protocol::model::Symbol;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    #[serde(rename = "public/respond-heartbeat")]
    Heartbeat,
    Subscribe,
    Unsubscribe,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Ticker,
    Trade,
    Book,
    #[serde(rename = "book.update")]
    Update,
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Channel::Ticker => write!(f, "ticker"),
            Channel::Trade => write!(f, "trade"),
            Channel::Book => write!(f, "book"),
            Channel::Update => write!(f, "book"),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ExchangeRequest {
    id: Option<i64>,
    method: Method,
    params: Option<Params>,
    nonce: i64,
}

#[derive(Serialize, Debug)]
pub struct Params {
    pub channels: Vec<String>,
    pub book_subscription_type: Option<String>,
}

impl Params {
    fn standard(channel: &Channel, market: &Market) -> Params {
        Params {
            channels: vec![format!("{}.{}", channel, market.exchange_format())],
            book_subscription_type: None,
        }
    }

    fn book(channel: &Channel, market: &Market) -> Params {
        Params {
            channels: vec![format!("{}.{}.10", channel, market.exchange_format())],
            book_subscription_type: Some("SNAPSHOT_AND_UPDATE".to_string()),
        }
    }
}

impl ExchangeRequest {
    pub fn heartbeat(id: i64) -> Self {
        Self {
            id: Some(id),
            method: Method::Heartbeat,
            params: None,
            nonce: Utc::now().timestamp_millis(),
        }
    }

    fn from_params(params: Params, method: Method) -> Self {
        Self {
            id: None,
            method,
            params: Some(params),
            nonce: Utc::now().timestamp_millis(),
        }
    }

    pub fn new(market: &Market, channel: &Channel, method: Method) -> Self {
        if *channel == Channel::Book {
            Self::from_params(Params::book(channel, market), method)
        } else {
            Self::from_params(Params::standard(channel, market), method)
        }
    }
}
