use crate::models::errors::HttpError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct OkResponse<T> {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub data: T,
    #[serde(skip_serializing)]
    code: StatusCode,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub data: Vec<HttpError>,
    #[serde(skip_serializing)]
    code: StatusCode,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.code, Json(self)).into_response()
    }
}

impl<T: Serialize> IntoResponse for OkResponse<T> {
    fn into_response(self) -> Response {
        (self.code, Json(self)).into_response()
    }
}

impl ErrorResponse {
    pub fn one(errors: HttpError, code: StatusCode) -> ErrorResponse {
        ErrorResponse {
            timestamp: Utc::now(),
            data: vec![errors],
            code,
        }
    }
}

impl<T> OkResponse<T> {
    pub fn new(data: T, code: StatusCode) -> OkResponse<T> {
        OkResponse {
            timestamp: Utc::now(),
            data,
            code,
        }
    }
}
