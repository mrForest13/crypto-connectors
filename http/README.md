# Http

Basic structures and traits for connectors http server. Include:
- healthcheck endpoint
- metrics endpoint
- healthcheck definition
- ok and error response definition
- server base route

## How to use it?

```rust
async fn run_server(config: &HttpConfig, service: HealthcheckService) -> Result<()> {
    let router = base_router(service);

    let listener: TcpListener = TcpListener::bind(config.address())
        .await
        .context("Error during server address binding")?;

    let server: () = axum::serve(listener, router)
        .await
        .context("Error during http server start")?;

    Ok(server)
}
```