use protocol::public::ticker::Tick;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Ticker {
    pub b: Decimal,
    pub bs: Decimal,
    pub k: Decimal,
    pub ks: Decimal,
    pub i: String,
    pub t: i64,
}

impl From<&Ticker> for Tick {
    fn from(ticker: &Ticker) -> Self {
        Tick {
            timestamp: ticker.t,
            ask_price: ticker.k.to_string(),
            ask_size: ticker.ks.to_string(),
            bid_price: ticker.b.to_string(),
            bid_size: ticker.bs.to_string(),
        }
    }
}
