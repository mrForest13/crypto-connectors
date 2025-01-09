use protocol::public::types::Exchange;
use protocol::topics::RequestTopic;

pub fn markets() -> RequestTopic {
    RequestTopic::markets(Exchange::Kraken)
}
