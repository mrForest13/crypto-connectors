use async_nats::client::PublishErrorKind;
use async_nats::header::IntoHeaderValue;
use async_nats::subject::ToSubject;
use async_nats::{
    ClientError, ConnectError, Event, HeaderMap, PublishError, Request, RequestError,
    RequestErrorKind, SubscribeError, Subscriber,
};
use async_nats::{HeaderValue, Message as NatsMessage};
use log::{info, warn};
use prost::bytes::Bytes;
use prost::Message;
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use strum_macros::Display;

pub const STATUS_HEADER: &str = "status";

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Ok,
    Error,
}

impl IntoHeaderValue for Status {
    fn into_header_value(self) -> HeaderValue {
        HeaderValue::from(self.to_string())
    }
}

impl Status {
    fn headers(self) -> HeaderMap {
        let mut headers: HeaderMap = HeaderMap::new();
        headers.insert(STATUS_HEADER, self);
        headers
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub host: String,
    pub port: u16,
    pub max_reconnects: usize,
}

impl NatsConfig {
    pub fn address(&self) -> String {
        format!("nats://{}:{}", self.host, self.port)
    }
}

pub struct NatsClient {
    client: async_nats::Client,
    status: Arc<RwLock<Event>>,
}

impl NatsClient {
    pub async fn new(config: &NatsConfig) -> Result<NatsClient, ConnectError> {
        info!("Starting new nats async client {}", config.address());

        let status: Arc<RwLock<Event>> = Arc::new(RwLock::new(Event::Connected));

        options(status.clone(), config.max_reconnects)
            .connect(config.address())
            .await
            .map(|client| NatsClient { client, status })
    }

    pub async fn queue_subscribe<S: ToSubject>(
        &self,
        subject: S,
        queue: String,
    ) -> Result<Subscriber, SubscribeError> {
        self.client.queue_subscribe(subject, queue).await
    }

    pub async fn subscribe<S: ToSubject>(&self, subject: S) -> Result<Subscriber, SubscribeError> {
        self.client.subscribe(subject).await
    }

    pub async fn send_message<T: Message, S: ToSubject>(
        &self,
        subject: S,
        message: T,
    ) -> Result<(), PublishError> {
        self.send(subject, message, Status::Ok).await
    }

    pub async fn send_error<T: Message, S: ToSubject>(
        &self,
        subject: S,
        message: T,
    ) -> Result<(), PublishError> {
        self.send(subject, message, Status::Error).await
    }

    async fn send<T: Message, S: ToSubject>(
        &self,
        subject: S,
        message: T,
        status: Status,
    ) -> Result<(), PublishError> {
        let headers: HeaderMap = status.headers();
        let mut buffer: Vec<u8> = Vec::new();

        let bytes = match message.encode(&mut buffer) {
            Ok(_) => Bytes::from(buffer),
            Err(error) => {
                warn!("Cannot encode message {}", error);
                return Err(PublishError::from(PublishErrorKind::Send));
            }
        };

        self.client
            .publish_with_headers(subject, headers, bytes)
            .await
    }

    pub async fn send_request<T: Message, S: ToSubject>(
        &self,
        subject: S,
        message: T,
    ) -> Result<NatsMessage, RequestError> {
        let mut buffer: Vec<u8> = Vec::new();

        let bytes = match message.encode(&mut buffer) {
            Ok(_) => Bytes::from(buffer),
            Err(error) => {
                warn!("Cannot encode message {}", error);
                return Err(RequestError::from(RequestErrorKind::Other));
            }
        };

        let request = Request::new().payload(bytes);

        self.client.send_request(subject, request).await
    }

    pub fn is_healthy(&self) -> bool {
        let status_guard = self.status.read().unwrap();
        *status_guard != Event::Closed
            && *status_guard != Event::ClientError(ClientError::MaxReconnects)
    }
}

fn options(status: Arc<RwLock<Event>>, max_reconnects: usize) -> async_nats::ConnectOptions {
    async_nats::ConnectOptions::new()
        .max_reconnects(max_reconnects)
        .event_callback(move |event| callback(status.clone(), event))
}

async fn callback(local: Arc<RwLock<Event>>, event: Event) {
    match local.write() {
        Ok(mut local) => {
            info!("Changing nats status to: {:?}", event);
            *local = event;
        }
        Err(error) => {
            warn!("Cannot update nats status: {:?}", error);
        }
    }
}
