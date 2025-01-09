use crate::decoder::{decode_message, parse_publish_error, parse_request_error};
use crate::subscription::NatsStream;
use protocol::client::NatsClient;
use protocol::model::Symbol;
use protocol::public::book::{OrderBookMessage, OrderBookRequest};
use protocol::public::error::ErrorMessage;
use protocol::public::market::{MarketType, MarketsMessage, MarketsRequest};
use protocol::public::ticker::{TickerMessage, TickerRequest};
use protocol::public::trade::{TradesMessage, TradesRequest};
use protocol::public::types::Exchange;
use protocol::topics::{RequestTopic, StreamTopic, Topic};

pub struct PublicConnector {
    client: NatsClient,
}

impl PublicConnector {
    pub fn new(client: NatsClient) -> Self {
        PublicConnector { client }
    }

    pub async fn markets<S: Symbol>(
        &self,
        exchange: Exchange,
        symbols: Vec<S>,
        market_type: MarketType,
    ) -> Result<MarketsMessage, ErrorMessage> {
        let market_type: Option<i32> = Some(market_type as i32);
        let topic: RequestTopic = RequestTopic::markets(exchange);
        let symbols: Vec<String> = symbols.iter().map(Symbol::nats_format).collect();

        let request: MarketsRequest = MarketsRequest {
            symbols,
            market_type,
        };

        self.client
            .send_request(topic, request)
            .await
            .map_err(parse_request_error)
            .and_then(decode_message)
    }

    pub async fn ticker<S: Symbol>(
        &self,
        exchange: Exchange,
        symbol: S,
    ) -> Result<NatsStream<TickerMessage>, ErrorMessage> {
        let topic: StreamTopic = StreamTopic::ticker(exchange, &symbol);
        let snapshot: TickerRequest = TickerRequest {};

        self.client
            .send_message(topic.snapshot(), snapshot)
            .await
            .map_err(parse_publish_error)?;

        NatsStream::new(&self.client, topic).await
    }

    pub async fn trades<S: Symbol>(
        &self,
        exchange: Exchange,
        symbol: S,
    ) -> Result<NatsStream<TradesMessage>, ErrorMessage> {
        let topic: StreamTopic = StreamTopic::trades(exchange, &symbol);
        let snapshot: TradesRequest = TradesRequest {};

        self.client
            .send_message(topic.snapshot(), snapshot)
            .await
            .map_err(parse_publish_error)?;

        NatsStream::new(&self.client, topic).await
    }

    pub async fn order_book<S: Symbol>(
        &self,
        exchange: Exchange,
        symbol: S,
    ) -> Result<NatsStream<OrderBookMessage>, ErrorMessage> {
        let topic: StreamTopic = StreamTopic::book(exchange, &symbol);
        let snapshot: OrderBookRequest = OrderBookRequest {};

        self.client
            .send_message(topic.snapshot(), snapshot)
            .await
            .map_err(parse_publish_error)?;

        NatsStream::new(&self.client, topic).await
    }
}
