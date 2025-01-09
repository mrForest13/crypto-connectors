use crate::book::models::{OrderBook, Pair, Update};
use crate::client::request::Channel;
use crate::model::Market;
use crate::topics;
use crate::utils::state::State;
use anyhow::anyhow;
use anyhow::Result;
use async_nats::subject::ToSubject;
use async_nats::Subject;
use chrono::Utc;
use protocol::public::book::{Book, Offer, OrderBookMessage};
use protocol::public::types::{Exchange, MessageType};
use rust_decimal::Decimal;
use std::cmp::Reverse;
use std::collections::BTreeMap;

pub struct OrderBookState {
    asks: BTreeMap<Decimal, Decimal>,
    bids: BTreeMap<Reverse<Decimal>, Decimal>,
    sequence: i64,
    last_update: Decimal,
    timestamp: i64,
}

impl Default for OrderBookState {
    fn default() -> Self {
        OrderBookState {
            asks: BTreeMap::default(),
            bids: BTreeMap::default(),
            sequence: -1,
            last_update: Decimal::NEGATIVE_ONE,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

impl OrderBookState {
    fn check_last_id(&self, update: Decimal) -> Result<()> {
        if self.last_update != Decimal::NEGATIVE_ONE || self.last_update == update {
            Ok(())
        } else {
            Err(anyhow!("Order book sequence id missed"))
        }
    }

    fn snapshot(
        &mut self,
        state: Book,
        u: Decimal,
        t: i64,
        asks: Vec<Pair>,
        bids: Vec<Pair>,
    ) -> Result<OrderBookMessage> {
        self.sequence = 0;
        self.last_update = u;
        self.timestamp = t;

        update_asks(&mut self.asks, asks);
        update_bids(&mut self.bids, bids);

        Ok(OrderBookMessage {
            r#type: MessageType::Snapshot as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            book: Some(state),
        })
    }

    fn update(
        &mut self,
        state: Book,
        u: Decimal,
        t: i64,
        pu: Decimal,
        update: Update,
    ) -> Result<OrderBookMessage> {
        self.check_last_id(pu)?;

        self.sequence += 1;
        self.timestamp = t;
        self.last_update = u;

        update_asks(&mut self.asks, update.asks);
        update_bids(&mut self.bids, update.bids);

        let message_type: MessageType = if self.sequence == 0 {
            MessageType::Snapshot
        } else {
            MessageType::Update
        };

        Ok(OrderBookMessage {
            r#type: message_type as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            book: Some(state),
        })
    }

    fn book(&self) -> Book {
        Book {
            asks: self
                .asks
                .iter()
                .map(|(rate, size)| Offer {
                    rate: rate.to_string(),
                    size: size.to_string(),
                })
                .collect(),
            bids: self
                .bids
                .iter()
                .map(|(rate, size)| Offer {
                    rate: rate.0.to_string(),
                    size: size.to_string(),
                })
                .collect(),
            timestamp: self.timestamp,
        }
    }
}

impl State<OrderBook, OrderBookMessage> for OrderBookState {
    fn update(&mut self, book: OrderBook) -> Result<OrderBookMessage> {
        let state: Book = Book::from(&book);
        match book {
            OrderBook::Snapshot { asks, bids, t, u } => self.snapshot(state, u, t, asks, bids),
            OrderBook::Update { update, t, u, pu } => self.update(state, u, t, pu, update),
        }
    }

    fn get(&self) -> OrderBookMessage {
        OrderBookMessage {
            r#type: MessageType::Snapshot as i32,
            sequence: self.sequence,
            exchange: Exchange::Cryptocom as i32,
            book: Some(self.book()),
        }
    }

    fn topic(&self, market: &Market) -> Subject {
        topics::order_book(market).to_subject()
    }

    fn channel(&self) -> Channel {
        Channel::Book
    }
}

fn update_asks(state: &mut BTreeMap<Decimal, Decimal>, new_offers: Vec<Pair>) {
    for offer in new_offers {
        if offer.size > Decimal::ZERO {
            state.insert(offer.rate, offer.size);
        } else {
            state.remove(&offer.rate);
        }
    }
}

fn update_bids(state: &mut BTreeMap<Reverse<Decimal>, Decimal>, new_offers: Vec<Pair>) {
    for offer in new_offers {
        if offer.size > Decimal::ZERO {
            state.insert(Reverse(offer.rate), offer.size);
        } else {
            state.remove(&Reverse(offer.rate));
        }
    }
}
