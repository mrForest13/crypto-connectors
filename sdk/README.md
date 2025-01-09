# Crypto Connectors SDK

Client for crypto connectors. Implement stream trait

## Api
- markets
- ticker
- trades
- order book

## Initialization

```rust
    let config: NatsConfig = NatsConfig {
        host: "0.0.0.0".to_string(),
        port: 4222,
        max_reconnects: 0,
    };

    let client: NatsClient = NatsClient::new(&config).await?;
    let connector: PublicConnector = PublicConnector::new(client);
```

## How to start?

check examples folder.