FROM rust:1.83 as build

# Protobuf instalation
RUN apt-get update && apt-get install -y protobuf-compiler

ARG CONNECTOR

RUN echo "Building docker image for ${CONNECTOR}"

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

COPY ./http/Cargo.toml ./http/Cargo.toml
COPY ./protocol/Cargo.toml ./protocol/Cargo.toml
COPY ./public-connector/Cargo.toml ./public-connector/Cargo.toml

COPY ./public-cryptocom/Cargo.toml ./public-cryptocom/Cargo.toml
COPY ./public-kraken/Cargo.toml ./public-kraken/Cargo.toml

COPY ./sdk/Cargo.toml ./sdk/Cargo.toml

RUN cargo fetch

# Copy the source code
COPY ./ ./

# Build for release.
RUN cargo build -p $CONNECTOR --release

RUN mv target/release/$CONNECTOR target/release/application
RUN mv $CONNECTOR/resources target/release/resources

# The final base image
FROM debian:bookworm-slim

# Certs instalation for websocket
RUN apt-get update && apt-get install -y libssl3 ca-certificates

ENV CONFIGURATION_PATH=configuration

COPY --from=build target/release/application /usr/src/application
COPY --from=build target/release/resources /configuration

CMD ["/usr/src/application"]
