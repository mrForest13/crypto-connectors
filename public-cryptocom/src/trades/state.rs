use crate::client::request::Channel;
use crate::model::Market;
use crate::topics;
use crate::trades::models::Transaction;
use crate::utils::state::State;
use anyhow::{anyhow, Result};
use async_nats::subject::ToSubject;
use async_nats::Subject;
use protocol::public::trade::{Trade, TradesMessage};
use protocol::public::types::{Exchange, MessageType};
use rust_decimal::Decimal;

const SNAPSHOT_SIZE: usize = 50;

pub struct TradesState {
    state: Vec<Trade>,
    last_id: Decimal,
    sequence: i64,
}

impl Default for TradesState {
    fn default() -> Self {
        TradesState {
            sequence: -1,
            last_id: Decimal::NEGATIVE_ONE,
            state: vec![],
        }
    }
}

impl TradesState {
    fn check_last_id(&self, id: Decimal) -> Result<()> {
        if self.last_id == Decimal::NEGATIVE_ONE || self.last_id + Decimal::ONE == id {
            Ok(())
        } else {
            Err(anyhow!("Transaction sequence id missed"))
        }
    }
}

impl State<Vec<Transaction>, TradesMessage> for TradesState {
    fn update(&mut self, trades: Vec<Transaction>) -> Result<TradesMessage> {
        if let Some(last_id) = trades.first().map(|tx| tx.m) {
            self.check_last_id(last_id)?;
            self.last_id = last_id;
        }

        let update: Vec<Trade> = convert(&trades);

        self.sequence += 1;
        self.state.splice(0..0, update.clone());

        self.state.truncate(SNAPSHOT_SIZE);

        let message_type: MessageType = if self.sequence == 0 {
            MessageType::Snapshot
        } else {
            MessageType::Update
        };

        Ok(TradesMessage {
            r#type: message_type as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            trades: update,
        })
    }

    fn get(&self) -> TradesMessage {
        TradesMessage {
            r#type: MessageType::Snapshot as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            trades: self.state.clone(),
        }
    }

    fn topic(&self, market: &Market) -> Subject {
        topics::trades(market).to_subject()
    }

    fn channel(&self) -> Channel {
        Channel::Trade
    }
}

fn convert(state: &[Transaction]) -> Vec<Trade> {
    state.iter().map(Trade::from).collect()
}
