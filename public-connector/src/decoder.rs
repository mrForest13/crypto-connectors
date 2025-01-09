use anyhow::anyhow;
use anyhow::Result;
use async_nats::{Message as NatsMessage, Subject};
use prost::Message as ProtoMessage;

const TOPIC_SEPARATOR: &str = ".";

pub struct NatsEvent<T> {
    pub message: T,
    pub subject: Subject,
    pub reply: Option<Subject>,
}

impl<T> NatsEvent<T> {
    /// Extracts the 'from' and 'to' symbols from the subject in the event.
    /// # Examples
    /// exchange.ticker.btc.usd.snapshot
    /// exchange.trades.btc.usd.snapshot
    /// exchange.book.btc.usd.snapshot
    pub fn symbols(&self) -> Result<(String, String)> {
        let subject: String = self.subject.to_string();
        let parts: Vec<&str> = subject.split(TOPIC_SEPARATOR).collect();

        if parts.len() >= 3 {
            let from: String = parts[2].to_string();
            let to: String = parts[3].to_string();

            Ok((from, to))
        } else {
            Err(anyhow!("Wrong topic format {}", self.subject.to_string()))
        }
    }
}

pub(crate) fn decode<T: ProtoMessage + Default>(message: NatsMessage) -> Result<NatsEvent<T>> {
    T::decode(message.payload)
        .map_err(|error| anyhow!("Nats message decode error: {}", error))
        .map(|proto| NatsEvent {
            message: proto,
            subject: message.subject,
            reply: message.reply,
        })
}
