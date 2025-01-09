# Crypto Connectors

The goal for project is providing crypto 
connectors with unify API (protobuf) as a service (docker images).

## Api

All streams and endpoints are using protocol buffers models. Check `protocol/proto`

## Public connector

Support markets configuration, order book, ticker and recent trades. 

## TODO list
- finish kraken connector
- add private connector (api based on api key) for both exchanges

## How to run it

1. Create images for cryptocom

`docker build --build-arg CONNECTOR=public-kraken -t public-kraken .`

2. Create images for kraken

`docker build --build-arg CONNECTOR=public-kraken -t public-kraken .`

3. Run test-compose.yml
4. Check examples `skd/examples`
