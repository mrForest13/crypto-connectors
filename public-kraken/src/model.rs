use anyhow::{anyhow, Result};
use protocol::model::{Currency, Symbol};
use serde::{de, Deserialize, Deserializer};

const KRAKEN_SEPARATOR: char = '/';

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

    pub fn from_string(market: String, separator: char) -> Result<Market> {
        let parts: Vec<&str> = market.split(separator).collect();

        let from: String = parts[0].to_string();
        let to: String = parts[1].to_string();

        if parts.len() == 2 {
            Ok(Market::new(from, to))
        } else {
            Err(anyhow!("Wrong market format: {}", market))
        }
    }

    pub fn from_exchange_format(market: String) -> Result<Market> {
        Market::from_string(market.to_string(), KRAKEN_SEPARATOR)
    }
}

impl Symbol for Market {
    fn from(&self) -> Currency {
        Currency::new(from_kraken_format(&self.from))
    }

    fn to(&self) -> Currency {
        Currency::new(from_kraken_format(&self.to))
    }

    fn exchange_format(&self) -> String {
        self.nats_format().to_uppercase()
    }
}

fn from_kraken_format(currency: &str) -> String {
    match currency {
        "xbt" => String::from("btc"),
        "xdg" => String::from("doge"),
        other => String::from(other),
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
