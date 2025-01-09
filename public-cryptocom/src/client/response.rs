use crate::client::request::Channel;
use crate::model::Market;
use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    #[serde(rename = "public/heartbeat")]
    Heartbeat,
    #[serde(rename = "public/get-instruments")]
    Instruments,
    Subscribe,
    Unsubscribe,
}

#[derive(Deserialize, Debug)]
pub struct ExchangeResponse<T> {
    pub id: i64,
    pub method: Method,
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct HttpResult<T> {
    pub data: Vec<T>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WsResult<T: Clone> {
    #[serde(rename = "instrument_name")]
    pub market: Market,
    pub channel: Channel,
    pub data: Vec<T>,
}

impl<T: Clone> WsResult<T> {
    pub fn update<G: Clone>(self, data: Vec<G>) -> WsResult<G> {
        WsResult {
            market: self.market,
            channel: self.channel,
            data,
        }
    }
}

impl<T: Clone> WsResult<T> {
    pub fn is_ticker(&self) -> bool {
        matches!(self.channel, Channel::Ticker)
    }

    pub fn is_trade(&self) -> bool {
        matches!(self.channel, Channel::Trade)
    }

    pub fn is_book(&self) -> bool {
        matches!(self.channel, Channel::Book | Channel::Update)
    }
}

#[derive(Deserialize, Debug)]
pub struct ExchangeError {
    pub code: i16,
    pub message: String,
}

impl Display for ExchangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Exchange error: {} with code {}",
            self.code, self.message
        )
    }
}

impl Error for ExchangeError {}
