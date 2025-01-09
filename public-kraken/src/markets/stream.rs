use crate::markets::handler::RequestHandler;

use std::sync::Arc;

use crate::config::ExchangeConfig;
use crate::topics;
use anyhow::Result;
use connector::decoder::NatsEvent;
use connector::http_client::HttpClient;
use connector::subscription::NatsSubscription;
use log::{debug, info, warn};
use protocol::client::NatsClient;
use protocol::public::market::MarketsRequest;
use protocol::topics::RequestTopic;
use tokio::sync::{OwnedSemaphorePermit as Permit, Semaphore};

const QUEUE: &str = "cryptocom.markets";

pub async fn run(
    nats_client: Arc<NatsClient>,
    http_client: Arc<HttpClient>,
    config: &ExchangeConfig,
) -> Result<()> {
    let topic: RequestTopic = topics::markets();

    info!("Starting markets request processing");

    let mut nats_subscription: NatsSubscription<MarketsRequest> =
        NatsSubscription::new(&nats_client, topic, QUEUE).await?;
    let request_handler: Arc<RequestHandler> =
        Arc::new(RequestHandler::new(http_client, nats_client, config)?);
    let limiter: Arc<Semaphore> = Arc::new(Semaphore::new(config.max_concurrency));

    while let Some(result) = nats_subscription.next().await {
        if let Ok(event) = result {
            let permit: Permit = limiter.clone().acquire_owned().await?;
            tokio::spawn(process(request_handler.clone(), event, permit));
        } else if let Err(error) = result {
            warn!("Cannot process nats message: {}", error)
        }
    }

    Ok(())
}

async fn process(handler: Arc<RequestHandler>, event: NatsEvent<MarketsRequest>, permit: Permit) {
    if let Err(error) = handler.process(event).await {
        warn!("Cannot send markets response: {}", error)
    } else {
        debug!("Markets response sent")
    }
    drop(permit);
}
