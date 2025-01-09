use protocol::public::market::{Market, MarketType};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::cmp::PartialEq;

#[derive(Deserialize, Debug, PartialEq)]
pub enum InstType {
    #[serde(rename = "CCY_PAIR")]
    CcyPair,
    #[serde(rename = "FUTURE")]
    Future,
    #[serde(rename = "PERPETUAL_SWAP")]
    PerpetualSwap,
}

impl From<&InstType> for MarketType {
    fn from(value: &InstType) -> Self {
        if value == &InstType::CcyPair {
            MarketType::Spot
        } else if value == &InstType::Future {
            MarketType::Future
        } else {
            MarketType::Perpetual
        }
    }
}

#[derive(Deserialize)]
pub struct Instrument {
    base_ccy: String,
    quote_ccy: String,
    inst_type: InstType,
    quote_decimals: i32,
    quantity_decimals: i32,
    price_tick_size: Decimal,
    qty_tick_size: Decimal,
    expiry_timestamp_ms: i64,
}

impl Instrument {
    fn symbol(&self) -> String {
        format!("{}_{}", self.base_ccy, self.quote_ccy)
    }

    fn expiry_timestamp(&self) -> Option<i64> {
        if self.expiry_timestamp_ms == 0 {
            None
        } else {
            Some(self.expiry_timestamp_ms)
        }
    }
}

impl From<&Instrument> for Market {
    fn from(instrument: &Instrument) -> Self {
        Market {
            symbol: instrument.symbol().to_lowercase(),
            price_precision: instrument.quote_decimals,
            rate_precision: instrument.quote_decimals,
            size_precision: instrument.quantity_decimals,
            min_size: instrument.qty_tick_size.to_string(),
            max_size: i64::MAX.to_string(),
            min_price: instrument.price_tick_size.to_string(),
            max_price: i64::MAX.to_string(),
            market_type: MarketType::from(&instrument.inst_type) as i32,
            expiry_timestamp: instrument.expiry_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::markets::models::InstType;
    use serde_json;
    use serde_json::from_str;

    #[test]
    fn deserialize_should_return_inst_type() {
        let ccy_pair = from_str(r#""CCY_PAIR""#);
        let future = from_str(r#""FUTURE""#);
        let swap = from_str(r#""PERPETUAL_SWAP""#);

        assert_eq!(ccy_pair.ok(), Some(InstType::CcyPair));
        assert_eq!(future.ok(), Some(InstType::Future));
        assert_eq!(swap.ok(), Some(InstType::PerpetualSwap));
    }
}
