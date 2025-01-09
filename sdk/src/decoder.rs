use async_nats::{HeaderMap, Message as NatsMessage, PublishError, SubscribeError};
use async_nats::{HeaderValue, RequestError};
use bytes::Bytes;
use chrono::Utc;
use prost::DecodeError;
use prost::Message as ProtoMessage;
use protocol::client;
use protocol::client::Status;
use protocol::public::error::{ErrorCode, ErrorMessage};

pub fn decode_message<T: ProtoMessage + Default>(message: NatsMessage) -> Result<T, ErrorMessage> {
    match decode_status(message.headers) {
        Status::Ok => decode_ok::<T>(message.payload),
        Status::Error => Err(decode_error(message.payload)),
    }
}

pub fn decode_ok<T: ProtoMessage + Default>(payload: Bytes) -> Result<T, ErrorMessage> {
    T::decode(payload).map_err(parse_decode_error)
}

pub fn decode_error(payload: Bytes) -> ErrorMessage {
    ErrorMessage::decode(payload).unwrap_or_else(parse_decode_error)
}

pub fn decode_status(headers: Option<HeaderMap>) -> Status {
    let ok: HeaderValue = HeaderValue::from(Status::Ok.to_string());

    headers
        .unwrap_or_default()
        .get(client::STATUS_HEADER)
        .filter(|status| **status == ok)
        .map(|_| Status::Ok)
        .unwrap_or(Status::Error)
}

pub fn parse_decode_error(error: DecodeError) -> ErrorMessage {
    ErrorMessage {
        code: ErrorCode::UnknownCode as i32,
        message: error.to_string(),
        exchange_message: None,
        timestamp: Utc::now().timestamp_millis(),
    }
}

pub fn parse_request_error(error: RequestError) -> ErrorMessage {
    ErrorMessage {
        code: ErrorCode::ConnectionRefused as i32,
        message: error.to_string(),
        exchange_message: None,
        timestamp: Utc::now().timestamp_millis(),
    }
}

pub fn parse_subscribe_error(error: SubscribeError) -> ErrorMessage {
    ErrorMessage {
        code: ErrorCode::ConnectionRefused as i32,
        message: error.to_string(),
        exchange_message: None,
        timestamp: Utc::now().timestamp_millis(),
    }
}

pub fn parse_publish_error(error: PublishError) -> ErrorMessage {
    ErrorMessage {
        code: ErrorCode::ConnectionRefused as i32,
        message: error.to_string(),
        exchange_message: None,
        timestamp: Utc::now().timestamp_millis(),
    }
}
