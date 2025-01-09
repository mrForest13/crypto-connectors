use crate::decoder::{decode, NatsEvent};
use anyhow::Result;
use async_nats::subject::ToSubject;
use async_nats::Subscriber;
use futures::StreamExt;
use prost::Message;
use protocol::client::NatsClient;
use tracing::info;

pub struct NatsSubscription<R: Message + Default> {
    subscriber: Subscriber,
    marker: std::marker::PhantomData<R>,
}

impl<R: Message + Default> NatsSubscription<R> {
    pub async fn new<T: ToSubject>(
        nats_client: &NatsClient,
        topic: T,
        queue: &str,
    ) -> Result<NatsSubscription<R>> {
        info!("Subscribe to nats topic {}", topic.to_subject());

        let subscriber: Subscriber = nats_client.queue_subscribe(topic, queue.into()).await?;

        Ok(NatsSubscription {
            subscriber,
            marker: std::marker::PhantomData,
        })
    }

    pub async fn next(&mut self) -> Option<Result<NatsEvent<R>>> {
        self.subscriber.next().await.map(decode::<R>)
    }
}
