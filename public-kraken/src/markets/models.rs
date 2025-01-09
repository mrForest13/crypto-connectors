use crate::model::Market;
use protocol::model::Symbol;
use protocol::public::market::MarketType;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AssetPair {
    wsname: Market,
    pair_decimals: i32,
    lot_decimals: i32,
    ordermin: Decimal,
    costmin: Decimal,
}

impl From<&AssetPair> for protocol::public::market::Market {
    fn from(pair: &AssetPair) -> Self {
        protocol::public::market::Market {
            symbol: pair.wsname.nats_format(),
            price_precision: pair.pair_decimals,
            rate_precision: pair.pair_decimals,
            size_precision: pair.lot_decimals,
            min_size: pair.ordermin.to_string(),
            max_size: i64::MAX.to_string(),
            min_price: pair.costmin.to_string(),
            max_price: i64::MAX.to_string(),
            market_type: MarketType::Spot as i32,
            expiry_timestamp: None,
        }
    }
}
