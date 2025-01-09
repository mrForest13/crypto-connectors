use connectors_sdk::connector::PublicConnector;
use protocol::client::{NatsClient, NatsConfig};
use protocol::model::{Currency, Symbol};
use protocol::public::market::{MarketType, MarketsMessage};
use protocol::public::types::Exchange;

#[tokio::main]
async fn main() {
    let exchange: Exchange = Exchange::Cryptocom;
    let config: NatsConfig = NatsConfig {
        host: "0.0.0.0".to_string(),
        port: 4222,
        max_reconnects: 0,
    };

    let client: NatsClient = NatsClient::new(&config).await.expect("Nats error");
    let connector: PublicConnector = PublicConnector::new(client);

    let symbols: Vec<Market> = vec![
        Market {
            from: "btc".into(),
            to: "eur".into(),
        },
        Market {
            from: "btc".into(),
            to: "usd".into(),
        },
    ];

    let response: MarketsMessage = connector
        .markets(exchange, symbols, MarketType::Spot)
        .await
        .unwrap();

    println!("{:?}", response);
}

pub struct Market {
    pub from: String,
    pub to: String,
}

impl Symbol for Market {
    fn from(&self) -> Currency {
        Currency::new(self.from.clone())
    }

    fn to(&self) -> Currency {
        Currency::new(self.to.clone())
    }

    fn exchange_format(&self) -> String {
        format!("{}-{}", self.from(), self.to()).to_uppercase()
    }
}
