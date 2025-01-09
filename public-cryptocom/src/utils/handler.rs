use crate::client::request::{Channel, ExchangeRequest, Method};
use crate::client::ws_client::WsClient;
use crate::config::ExchangeConfig;
use crate::model::Market;
use crate::utils::state::State;
use anyhow::{anyhow, Result};
use async_nats::Subject;
use log::{info, warn};
use prost::Message;
use protocol::client::NatsClient;
use protocol::model::Symbol;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Deserialize, Debug, Clone)]
pub enum Event<T> {
    Get(Market),
    Updated(Market, T),
}

impl<T> Event<T> {
    fn market(&self) -> Market {
        match self {
            Self::Get(market) => market.clone(),
            Self::Updated(market, _) => market.clone(),
        }
    }
}

pub struct Handler<T> {
    nats_client: Arc<NatsClient>,
    ws_client: Arc<WsClient>,
    state: HashMap<Market, Sender<Event<T>>>,
    receiver: Receiver<Event<T>>,
    buffer_size: usize,
}

impl<T: Send + 'static> Handler<T> {
    pub fn new(
        nats_client: Arc<NatsClient>,
        ws_client: Arc<WsClient>,
        config: &ExchangeConfig,
    ) -> (Self, Sender<Event<T>>) {
        let (sender, receiver): (Sender<Event<T>>, Receiver<Event<T>>) =
            channel::<Event<T>>(config.max_buffer_size);

        let handler: Handler<T> = Handler {
            nats_client,
            ws_client,
            buffer_size: config.max_buffer_size,
            state: HashMap::new(),
            receiver,
        };

        (handler, sender)
    }

    pub async fn run<M: Message, S: State<T, M>>(
        mut self,
        mut shutdown: broadcast::Receiver<()>,
    ) -> Result<()> {
        loop {
            select! {
                Some(event) = self.receiver.recv() => {
                    self.process::<M, S>(event).await?
                },
                Ok(_) = shutdown.recv() => {
                    warn!("Closing all processors!");
                    self.state.clear()
                }
            }
        }
    }

    async fn process<M: Message, S: State<T, M>>(&mut self, event: Event<T>) -> Result<()> {
        let market: Market = event.market();
        if let Some(sender) = self.state.get(&market) {
            sender
                .send(event)
                .await
                .map_err(|_| anyhow!("Cannot send event"))
        } else {
            let nats_client: Arc<NatsClient> = self.nats_client.clone();
            let ws_client: Arc<WsClient> = self.ws_client.clone();
            let (sender, receiver): (Sender<Event<T>>, Receiver<Event<T>>) =
                channel::<Event<T>>(self.buffer_size);

            let state: S = S::default();
            let channel: Channel = state.channel();
            let subscribe: ExchangeRequest = subscribe(&market, &channel);
            let unsubscribe: ExchangeRequest = unsubscribe(&market, &channel);

            ws_client.send(subscribe)?;
            self.state.insert(market.clone(), sender.clone());

            tokio::spawn(async move {
                let state: S = S::default();
                run_handler::<T, M, S>(nats_client, state, receiver, &market).await;
                ws_client.send(unsubscribe).unwrap_or_default();
            });

            Ok(())
        }
    }
}

fn subscribe(market: &Market, channel: &Channel) -> ExchangeRequest {
    ExchangeRequest::new(market, channel, Method::Subscribe)
}

fn unsubscribe(market: &Market, channel: &Channel) -> ExchangeRequest {
    ExchangeRequest::new(market, channel, Method::Unsubscribe)
}

async fn run_handler<T, M: Message, S: State<T, M>>(
    nats_client: Arc<NatsClient>,
    mut state: S,
    mut handler: Receiver<Event<T>>,
    market: &Market,
) {
    info!(
        "Running new {} task for {}",
        state.channel(),
        market.nats_format()
    );

    while let Some(event) = handler.recv().await {
        let topic: Subject = state.topic(market);

        let message: M = match state.publish(event) {
            Ok(message) => message,
            Err(error) => {
                warn!(
                    "Closing task {} for {}: {}",
                    state.channel(),
                    market.nats_format(),
                    error
                );
                break;
            }
        };

        if let Err(error) = nats_client.send_message(topic, message).await {
            warn!(
                "Closing task {} for {}: {}",
                state.channel(),
                market.nats_format(),
                error
            );
            break;
        }
    }
}
