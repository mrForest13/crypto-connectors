use crate::model::{Currency, Symbol};
use crate::public::types::Exchange;
use async_nats::subject::ToSubject;
use async_nats::Subject;
use strum_macros::Display;

/// Enum for topic creation
/// {exchange}.{endpoint}.btc.usd
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
enum Endpoint {
    Markets,
    Ticker,
    Trades,
    Book,
}

pub trait Topic: ToSubject {
    fn snapshot(&self) -> SnapshotTopic {
        SnapshotTopic {
            topic: self.to_subject(),
        }
    }
}

pub struct RequestTopic {
    exchange: Exchange,
    endpoint: Endpoint,
}

pub struct StreamTopic {
    exchange: Exchange,
    endpoint: Endpoint,
    from: Currency,
    to: Currency,
}

pub struct SnapshotTopic {
    topic: Subject,
}

impl ToSubject for SnapshotTopic {
    fn to_subject(&self) -> Subject {
        Subject::from(format!("{}.{}", self.topic, "snapshot"))
    }
}

impl RequestTopic {
    pub fn markets(exchange: Exchange) -> RequestTopic {
        RequestTopic {
            exchange,
            endpoint: Endpoint::Markets,
        }
    }
}

impl ToSubject for RequestTopic {
    fn to_subject(&self) -> Subject {
        Subject::from(format!(
            "{}.{}",
            self.exchange.as_str_name().to_lowercase(),
            self.endpoint
        ))
    }
}

impl Topic for RequestTopic {}

impl StreamTopic {
    fn new<S: Symbol>(exchange: Exchange, endpoint: Endpoint, symbol: &S) -> StreamTopic {
        StreamTopic {
            exchange,
            endpoint,
            from: symbol.from(),
            to: symbol.to(),
        }
    }

    pub fn ticker<S: Symbol>(exchange: Exchange, symbol: &S) -> StreamTopic {
        StreamTopic::new(exchange, Endpoint::Ticker, symbol)
    }

    pub fn trades<S: Symbol>(exchange: Exchange, symbol: &S) -> StreamTopic {
        StreamTopic::new(exchange, Endpoint::Trades, symbol)
    }

    pub fn book<S: Symbol>(exchange: Exchange, symbol: &S) -> StreamTopic {
        StreamTopic::new(exchange, Endpoint::Book, symbol)
    }
}

impl ToSubject for StreamTopic {
    fn to_subject(&self) -> Subject {
        Subject::from(format!(
            "{}.{}.{}.{}",
            self.exchange.as_str_name().to_lowercase(),
            self.endpoint,
            self.from,
            self.to
        ))
    }
}

impl Topic for StreamTopic {}

#[cfg(test)]
mod tests {
    mod markets {
        use crate::topics::{Exchange, RequestTopic};
        use async_nats::subject::ToSubject;

        #[test]
        fn market_topic_should_return_cryptocom_markets() {
            let exchange: Exchange = Exchange::Cryptocom;
            let topic: RequestTopic = RequestTopic::markets(exchange);

            let expected: &str = "cryptocom.markets";

            assert_eq!(topic.to_subject().as_str(), expected);
        }
    }

    mod ticker {
        use crate::tests::TestMarket;
        use crate::topics::{Exchange, StreamTopic, Topic};
        use async_nats::subject::ToSubject;

        #[test]
        fn ticker_topic_should_return_cryptocom_ticker_btc_usd_snapshot() {
            let from: String = "BTC".to_string();
            let to: String = "EUR".to_string();

            let exchange: Exchange = Exchange::Cryptocom;
            let symbol: TestMarket = TestMarket { from, to };
            let topic: StreamTopic = StreamTopic::ticker(exchange, &symbol);

            let expected: &str = "cryptocom.ticker.btc.eur.snapshot";

            assert_eq!(topic.snapshot().to_subject().as_str(), expected);
        }
    }

    mod trades {
        use crate::tests::TestMarket;
        use crate::topics::{Exchange, StreamTopic, Topic};
        use async_nats::subject::ToSubject;

        #[test]
        fn trades_topic_should_return_cryptocom_trades_btc_usd() {
            let from: String = "BTC".to_string();
            let to: String = "EUR".to_string();

            let exchange: Exchange = Exchange::Cryptocom;
            let symbol: TestMarket = TestMarket { from, to };
            let topic: StreamTopic = StreamTopic::trades(exchange, &symbol);

            let expected: &str = "cryptocom.trades.btc.eur";

            assert_eq!(topic.to_subject().as_str(), expected);
        }

        #[test]
        fn trades_topic_should_return_cryptocom_trades_btc_usd_snapshot() {
            let from: String = "BTC".to_string();
            let to: String = "EUR".to_string();

            let exchange: Exchange = Exchange::Cryptocom;
            let symbol: TestMarket = TestMarket { from, to };
            let topic: StreamTopic = StreamTopic::trades(exchange, &symbol);

            let expected: &str = "cryptocom.trades.btc.eur.snapshot";

            assert_eq!(topic.snapshot().to_subject().as_str(), expected);
        }
    }

    mod book {
        use crate::tests::TestMarket;
        use crate::topics::{Exchange, StreamTopic, Topic};
        use async_nats::subject::ToSubject;

        #[test]
        fn order_book_topic_should_return_cryptocom_book_btc_usd() {
            let from: String = "BTC".to_string();
            let to: String = "EUR".to_string();

            let exchange: Exchange = Exchange::Cryptocom;
            let symbol: TestMarket = TestMarket { from, to };
            let topic: StreamTopic = StreamTopic::book(exchange, &symbol);

            let expected: &str = "cryptocom.book.btc.eur";

            assert_eq!(topic.to_subject().as_str(), expected);
        }

        #[test]
        fn order_book_topic_should_return_cryptocom_book_btc_usd_snapshot() {
            let from: String = "BTC".to_string();
            let to: String = "EUR".to_string();

            let exchange: Exchange = Exchange::Cryptocom;
            let symbol: TestMarket = TestMarket { from, to };
            let topic: StreamTopic = StreamTopic::book(exchange, &symbol);

            let expected: &str = "cryptocom.book.btc.eur.snapshot";

            assert_eq!(topic.snapshot().to_subject().as_str(), expected);
        }
    }
}
