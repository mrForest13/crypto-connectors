use protocol::public::book::{Book, Offer};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OrderBook {
    Snapshot {
        asks: Vec<Pair>,
        bids: Vec<Pair>,
        t: i64,
        u: Decimal,
    },
    Update {
        update: Update,
        t: i64,
        u: Decimal,
        pu: Decimal,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct Update {
    pub asks: Vec<Pair>,
    pub bids: Vec<Pair>,
}

#[derive(Clone, Debug)]
pub struct Pair {
    pub rate: Decimal,
    pub size: Decimal,
}

impl Pair {
    fn offer(&self) -> Offer {
        Offer {
            size: self.size.to_string(),
            rate: self.rate.to_string(),
        }
    }
}

impl From<&OrderBook> for Book {
    fn from(book: &OrderBook) -> Self {
        match book {
            OrderBook::Snapshot {
                asks,
                bids,
                t,
                u: _,
            } => Book {
                asks: asks.iter().map(Pair::offer).collect(),
                bids: bids.iter().map(Pair::offer).collect(),
                timestamp: *t,
            },
            OrderBook::Update {
                update,
                t,
                u: _,
                pu: _,
            } => Book {
                asks: update.asks.iter().map(Pair::offer).collect(),
                bids: update.bids.iter().map(Pair::offer).collect(),
                timestamp: *t,
            },
        }
    }
}

impl<'de> Deserialize<'de> for Pair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vector: Vec<String> = Deserialize::deserialize(deserializer)?;

        if vector.len() == 3 {
            let rate: Decimal = vector[0]
                .parse::<Decimal>()
                .map_err(serde::de::Error::custom)?;
            let size: Decimal = vector[1]
                .parse::<Decimal>()
                .map_err(serde::de::Error::custom)?;

            Ok(Pair { rate, size })
        } else {
            Err(serde::de::Error::custom("Expected array of length 3"))
        }
    }
}
