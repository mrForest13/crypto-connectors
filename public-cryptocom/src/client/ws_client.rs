use crate::book::models::OrderBook;
use crate::client::request::ExchangeRequest;
use crate::client::response::{ExchangeResponse, Method, WsResult};
use crate::config::ExchangeConfig;
use crate::ticker::models::Ticker;
use crate::trades::models::Transaction;
use anyhow::{anyhow, Result};
use futures::stream::SplitSink;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type Event = ExchangeResponse<Option<WsResult<Value>>>;
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

type WsSender<T> = Sender<WsResult<T>>;
type WsReceiver<T> = Receiver<WsResult<T>>;

pub struct WsClient {
    ws_uri: Uri,
    channels_in: ChannelsIn,
    channels_out: ChannelsOut,
}

#[derive(Clone)]
struct ChannelsIn {
    message_in: Sender<Message>,
    tickers_in: Sender<WsResult<Ticker>>,
    trades_in: Sender<WsResult<Transaction>>,
    books_in: Sender<WsResult<OrderBook>>,
    shutdown_in: Sender<()>,
}

struct ChannelsOut {
    message_out: Receiver<Message>,
    tickers_out: Receiver<WsResult<Ticker>>,
    trades_out: Receiver<WsResult<Transaction>>,
    books_out: Receiver<WsResult<OrderBook>>,
    shutdown_out: Receiver<()>,
}

impl WsClient {
    pub fn new(config: &ExchangeConfig) -> Result<WsClient> {
        let ws_uri: Uri = Uri::from_str(config.ws_url.as_str())?;
        let size: usize = config.max_buffer_size;

        let (shutdown_in, shutdown_out): (Sender<()>, Receiver<()>) =
            broadcast::channel::<()>(size);
        let (message_in, message_out): (Sender<Message>, Receiver<Message>) =
            broadcast::channel::<Message>(size);
        let (tickers_in, tickers_out): (WsSender<Ticker>, WsReceiver<Ticker>) =
            broadcast::channel::<WsResult<Ticker>>(size);
        let (trades_in, trades_out): (WsSender<Transaction>, WsReceiver<Transaction>) =
            broadcast::channel::<WsResult<Transaction>>(size);
        let (books_in, books_out): (WsSender<OrderBook>, WsReceiver<OrderBook>) =
            broadcast::channel::<WsResult<OrderBook>>(size);

        let channels_in = ChannelsIn {
            message_in,
            tickers_in,
            trades_in,
            books_in,
            shutdown_in,
        };
        let channels_out = ChannelsOut {
            message_out,
            tickers_out,
            trades_out,
            books_out,
            shutdown_out,
        };

        Ok(WsClient {
            ws_uri,
            channels_in,
            channels_out,
        })
    }

    pub fn send(&self, request: ExchangeRequest) -> Result<()> {
        self.channels_in.send_json(request)
    }

    fn subscribe_message(&self) -> Receiver<Message> {
        self.channels_out.message_out.resubscribe()
    }

    pub fn subscribe_shutdown(&self) -> Receiver<()> {
        self.channels_out.shutdown_out.resubscribe()
    }

    pub fn subscribe_book(&self) -> Receiver<WsResult<OrderBook>> {
        self.channels_out.books_out.resubscribe()
    }

    pub fn subscribe_ticker(&self) -> Receiver<WsResult<Ticker>> {
        self.channels_out.tickers_out.resubscribe()
    }

    pub fn subscribe_trade(&self) -> Receiver<WsResult<Transaction>> {
        self.channels_out.trades_out.resubscribe()
    }

    pub async fn run(&self) -> Result<()> {
        loop {
            let ws_uri: &Uri = &self.ws_uri;
            let channels_in: &ChannelsIn = &self.channels_in;
            let message_out: Receiver<Message> = self.subscribe_message();

            if let Err(error) = connect(ws_uri, channels_in, message_out).await {
                warn!("Websocket restarting on error: {}", error);
            }
        }
    }
}

async fn connect(
    uri: &Uri,
    channels: &ChannelsIn,
    mut message_out: Receiver<Message>,
) -> Result<()> {
    let (ws_stream, _): (WsStream, Response) = connect_async(uri).await?;

    let (mut sink, mut stream): (SplitSink<WsStream, Message>, SplitStream<WsStream>) =
        ws_stream.split();

    info!("WebSocket connection established {}!", uri.to_string());

    tokio::spawn(async move {
        while let Ok(result) = message_out.recv().await {
            if let Err(error) = sink.send(result).await {
                warn!("Websocket connection already closed: {}", error);
                break;
            }
        }
    });

    while let Some(message) = stream.next().await {
        let result: Result<()> = match message {
            Ok(Message::Text(json)) => {
                debug!("Processing ws message: {}", json);
                process_event(json, channels)
            }
            Ok(Message::Close(_)) => {
                warn!("Websocket connection closed by client!");
                Err(anyhow!(Error::ConnectionClosed))
            }
            Ok(Message::Ping(data)) => {
                debug!("Processing ping message");
                channels.send_message(Message::Pong(data))
            }
            Ok(_) => {
                warn!("Unsupported message type!");
                Err(anyhow!(Error::ConnectionClosed))
            }
            Err(error) => {
                warn!("WebSocket connection error: {}", error);
                Err(anyhow!(error))
            }
        };

        if let Err(error) = result {
            channels.shutdown_in.send(())?;
            return Err(error);
        }
    }

    Ok(())
}

impl ChannelsIn {
    fn send_json<T: Serialize>(&self, message: T) -> Result<()> {
        let json: String = serde_json::to_string(&message)?;
        self.send_message(Message::text(json))
    }

    fn send_message(&self, message: Message) -> Result<()> {
        self.message_in
            .send(message)
            .map(|_| ())
            .map_err(|error| anyhow!(error))
    }

    fn send_ticker(&self, message: WsResult<Ticker>) -> Result<()> {
        self.tickers_in
            .send(message)
            .map(|_| ())
            .map_err(|error| anyhow!(error))
    }

    fn send_trade(&self, message: WsResult<Transaction>) -> Result<()> {
        self.trades_in
            .send(message)
            .map(|_| ())
            .map_err(|error| anyhow!(error))
    }

    fn send_book(&self, message: WsResult<OrderBook>) -> Result<()> {
        self.books_in
            .send(message)
            .map(|_| ())
            .map_err(|error| anyhow!(error))
    }
}

fn process_event(json: String, channels: &ChannelsIn) -> Result<()> {
    match from_string::<Event>(&json) {
        Ok(ExchangeResponse {
            id,
            method: Method::Heartbeat,
            result: _,
        }) => channels.send_json(ExchangeRequest::heartbeat(id)),
        Ok(ExchangeResponse {
            id: _,
            method: Method::Subscribe,
            result: Some(result),
        }) if result.is_ticker() => {
            let tickers: Vec<Ticker> = from_value(&result.data)?;
            let event: WsResult<Ticker> = result.update(tickers);
            channels.send_ticker(event)
        }
        Ok(ExchangeResponse {
            id: _,
            method: Method::Subscribe,
            result: Some(result),
        }) if result.is_trade() => {
            let trades: Vec<Transaction> = from_value(&result.data)?;
            let event: WsResult<Transaction> = result.update(trades);
            channels.send_trade(event)
        }
        Ok(ExchangeResponse {
            id: _,
            method: Method::Subscribe,
            result: Some(result),
        }) if result.is_book() => {
            let book: Vec<OrderBook> = from_value(&result.data)?;
            let event: WsResult<OrderBook> = result.update(book);
            channels.send_book(event)
        }
        Ok(ExchangeResponse {
            id: _,
            method: Method::Unsubscribe,
            result: _,
        }) => {
            debug!("Unsubscribe: {}", json);
            Ok(())
        }
        Ok(ExchangeResponse {
            id: _,
            method: _,
            result: _,
        }) => {
            warn!("Invalid json format {}", json);
            Ok(())
        }
        Err(error) => Err(anyhow!(error)),
    }
}

fn from_value<T: DeserializeOwned>(json: &[Value]) -> Result<Vec<T>> {
    let array: Value = Value::Array(json.to_vec());
    serde_json::from_value::<Vec<T>>(array).map_err(|error| anyhow!(error))
}

fn from_string<T: DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str::<T>(json).map_err(|error| anyhow!(error))
}
