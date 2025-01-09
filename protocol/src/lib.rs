pub mod client;
pub mod model;
pub mod topics;

pub mod public {

    pub mod error {
        include!(concat!(env!("OUT_DIR"), "/error.rs"));
    }

    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/types.rs"));
    }

    pub mod book {
        include!(concat!(env!("OUT_DIR"), "/book.rs"));
    }

    pub mod market {
        include!(concat!(env!("OUT_DIR"), "/market.rs"));
    }

    pub mod ticker {
        include!(concat!(env!("OUT_DIR"), "/ticker.rs"));
    }

    pub mod trade {
        include!(concat!(env!("OUT_DIR"), "/trade.rs"));
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{Currency, Symbol};

    pub struct TestMarket {
        pub from: String,
        pub to: String,
    }

    impl Symbol for TestMarket {
        fn from(&self) -> Currency {
            Currency::new(self.from.clone())
        }

        fn to(&self) -> Currency {
            Currency::new(self.to.clone())
        }

        fn exchange_format(&self) -> String {
            format!("{}-{}", self.from(), self.to()).to_uppercase()
        }
    }
}
