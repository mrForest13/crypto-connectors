use crate::client::request::Channel;
use crate::model::Market;
use crate::ticker::models::Ticker;
use crate::topics;
use crate::utils::state::State;
use anyhow::Result;
use async_nats::subject::ToSubject;
use async_nats::Subject;
use protocol::public::ticker::{Tick, TickerMessage};
use protocol::public::types::{Exchange, MessageType};

pub struct TickerState {
    state: Tick,
    sequence: i64,
}

impl Default for TickerState {
    fn default() -> Self {
        TickerState {
            sequence: -1,
            state: Tick::default(),
        }
    }
}

impl State<Ticker, TickerMessage> for TickerState {
    fn update(&mut self, dto: Ticker) -> Result<TickerMessage> {
        self.sequence += 1;
        self.state = Tick::from(&dto);

        let message_type: MessageType = if self.sequence == 0 {
            MessageType::Snapshot
        } else {
            MessageType::Update
        };

        Ok(TickerMessage {
            r#type: message_type as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            tick: Some(self.state.clone()),
        })
    }

    fn get(&self) -> TickerMessage {
        TickerMessage {
            r#type: MessageType::Snapshot as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            tick: Some(self.state.clone()),
        }
    }

    fn topic(&self, market: &Market) -> Subject {
        topics::ticker(market).to_subject()
    }

    fn channel(&self) -> Channel {
        Channel::Ticker
    }
}
