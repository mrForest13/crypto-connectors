use crate::model::Market;
use crate::utils::handler::Event;
use crate::utils::handler::Event::Get;
use connector::decoder::NatsEvent;
use connector::subscription::NatsSubscription;
use log::warn;
use prost::Message;
use tokio::sync::mpsc::Sender;

pub async fn handle_nats_subscription<E: Send + Sync + 'static, R: Message + Default>(
    events: Sender<Event<E>>,
    mut subscription: NatsSubscription<R>,
) -> anyhow::Result<()> {
    while let Some(result) = subscription.next().await {
        if let Err(error) = result {
            warn!("Cannot process nats message: {}", error)
        } else if let Ok(snapshot) = result.and_then(get) {
            events.send(snapshot).await?;
        }
    }

    Ok(())
}

pub fn get<E, R: Message>(event: NatsEvent<R>) -> anyhow::Result<Event<E>> {
    event
        .symbols()
        .map(|(from, to)| Market::new(from, to))
        .map(Get)
}
