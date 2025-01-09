use protocol::public::trade::Trade;
use protocol::public::types::Side;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradeSide {
    Sell,
    Buy,
}

impl From<&TradeSide> for Side {
    fn from(value: &TradeSide) -> Self {
        if value == &TradeSide::Sell {
            Side::Sell
        } else {
            Side::Buy
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    pub d: Decimal,
    pub p: Decimal,
    pub q: Decimal,
    pub s: TradeSide,
    pub t: i64,
    pub m: Decimal,
}

impl From<&Transaction> for Trade {
    fn from(ticker: &Transaction) -> Self {
        Trade {
            timestamp: ticker.t,
            id: ticker.d.to_string(),
            rate: ticker.p.to_string(),
            size: ticker.q.to_string(),
            side: Side::from(&ticker.s) as i32,
        }
    }
}
