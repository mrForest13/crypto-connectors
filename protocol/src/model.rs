use std::fmt::{Display, Formatter};

pub struct Currency {
    symbol: String,
}

impl Currency {
    pub fn new(symbol: String) -> Currency {
        Currency {
            symbol: symbol.to_lowercase(),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

/// Definition for pair e.g. BTC-EUR
/// nats_format (btc_usd) is used by all proto models
pub trait Symbol {
    fn from(&self) -> Currency;
    fn to(&self) -> Currency;

    fn nats_format(&self) -> String {
        format!("{}_{}", self.from(), self.to())
    }

    fn exchange_format(&self) -> String;
}

#[cfg(test)]
mod tests {
    use crate::model::Symbol;
    use crate::tests::TestMarket;

    #[test]
    fn nats_format_should_return_lowercase_from_to() {
        let from: String = "BTC".to_string();
        let to: String = "EUR".to_string();

        let symbol: TestMarket = TestMarket { from, to };

        assert_eq!(symbol.nats_format(), "btc_eur");
    }

    #[test]
    fn exchange_format_should_return_uppercase_from_to() {
        let from: String = "BTC".to_string();
        let to: String = "EUR".to_string();

        let symbol: TestMarket = TestMarket { from, to };

        assert_eq!(symbol.exchange_format(), "BTC-EUR");
    }
}
