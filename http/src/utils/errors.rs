use crate::models::errors::{ErrorCode, HttpError};
use crate::models::response::ErrorResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub fn page_not_found() -> HttpError {
    HttpError {
        code: ErrorCode::NotFound,
        message: String::from("Page not found"),
    }
}

pub fn unavailable() -> HttpError {
    HttpError {
        code: ErrorCode::Unavailable,
        message: "One of the services is unavailable!".to_string(),
    }
}

pub async fn not_found_handler() -> impl IntoResponse {
    ErrorResponse::one(page_not_found(), StatusCode::NOT_FOUND)
}
