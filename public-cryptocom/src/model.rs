use anyhow::{anyhow, Result};
use protocol::model::{Currency, Symbol};
use serde::{de, Deserialize, Deserializer};

const NATS_SEPARATOR: char = '_';

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct Market {
    from: String,
    to: String,
}

impl Market {
    pub fn new(from: String, to: String) -> Self {
        Self {
            from: from.to_lowercase(),
            to: to.to_lowercase(),
        }
    }

    pub fn from_nats_format(market: String) -> Result<Market> {
        let parts: Vec<&str> = market.split(NATS_SEPARATOR).collect();

        let from: String = parts[0].to_string();
        let to: String = parts[1].to_string();

        if parts.len() == 2 {
            Ok(Market::new(from, to))
        } else {
            Err(anyhow!("Wrong market format: {}", market))
        }
    }

    pub fn from_exchange_format(market: String) -> Result<Market> {
        Market::from_nats_format(market.to_string())
    }
}

impl Symbol for Market {
    fn from(&self) -> Currency {
        Currency::new(self.from.clone())
    }

    fn to(&self) -> Currency {
        Currency::new(self.to.clone())
    }

    fn exchange_format(&self) -> String {
        self.nats_format().to_uppercase()
    }
}

impl<'de> Deserialize<'de> for Market {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;

        Market::from_exchange_format(value).map_err(de::Error::custom)
    }
}
