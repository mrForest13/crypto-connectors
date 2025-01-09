use crate::model::Market;
use protocol::public::types::Exchange;
use protocol::topics::{RequestTopic, StreamTopic};

pub fn markets() -> RequestTopic {
    RequestTopic::markets(Exchange::Cryptocom)
}

pub fn ticker(symbol: &Market) -> StreamTopic {
    StreamTopic::ticker(Exchange::Cryptocom, symbol)
}

pub fn trades(symbol: &Market) -> StreamTopic {
    StreamTopic::trades(Exchange::Cryptocom, symbol)
}

pub fn order_book(symbol: &Market) -> StreamTopic {
    StreamTopic::book(Exchange::Cryptocom, symbol)
}
