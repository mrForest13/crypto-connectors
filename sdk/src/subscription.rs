use crate::decoder::parse_subscribe_error;
use async_nats::subject::ToSubject;
use async_nats::Subscriber;
use futures::{Stream, StreamExt};
use log::{error, info};
use prost::Message as ProtoMessage;
use protocol::client::NatsClient;
use protocol::public::error::ErrorMessage;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct NatsStream<E> {
    receiver: Receiver<E>,
}

impl<E: ProtoMessage + Default + 'static> NatsStream<E> {
    pub async fn new<T: ToSubject>(
        nats_client: &NatsClient,
        topic: T,
    ) -> Result<Self, ErrorMessage> {
        info!("Subscribe to nats topic {}", topic.to_subject());

        let (sender, receiver): (Sender<E>, Receiver<E>) = mpsc::channel::<E>(100);

        let mut subscriber: Subscriber = nats_client
            .subscribe(topic)
            .await
            .map_err(parse_subscribe_error)?;

        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                let event: E = match E::decode(message.payload) {
                    Ok(event) => event,
                    Err(error) => {
                        error!("Cannot decode message {}", error);
                        drop(sender);
                        break;
                    }
                };

                match sender.send(event).await {
                    Ok(_) => {}
                    Err(error) => {
                        error!("Cannot publish message {}", error);
                        drop(sender);
                        break;
                    }
                }
            }
        });

        Ok(NatsStream { receiver })
    }
}

impl<T> Stream for NatsStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
