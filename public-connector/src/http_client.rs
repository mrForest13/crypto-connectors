use chrono::Utc;
use protocol::public::error::{ErrorCode, ErrorMessage};
use reqwest::{Error, Response, Url};
use serde::de::DeserializeOwned;
use std::error::Error as StdError;
use tracing::{info, warn};

pub struct HttpClient {
    client: reqwest::Client,
}

impl Default for HttpClient {
    fn default() -> Self {
        info!("Starting new http client");
        HttpClient {
            client: reqwest::Client::new(),
        }
    }
}

impl HttpClient {
    pub async fn get<T: DeserializeOwned, E: DeserializeOwned + StdError>(
        &self,
        url: &Url,
    ) -> Result<T, ErrorMessage> {
        let response: Result<Response, Error> = self.client.get(url.clone()).send().await;

        match response {
            Ok(payload) => decode::<T, E>(payload).await,
            Err(error) => Err(from_error(error)),
        }
    }
}

async fn decode<T: DeserializeOwned, E: DeserializeOwned + StdError>(
    response: Response,
) -> Result<T, ErrorMessage> {
    if response.status().is_success() {
        match response.json::<T>().await {
            Ok(data) => Ok(data),
            Err(error) => Err(from_error(error)),
        }
    } else {
        match response.json::<E>().await {
            Ok(error) => Err(from_error(error)),
            Err(error) => Err(from_error(error)),
        }
    }
}

fn from_error<E: StdError>(error: E) -> ErrorMessage {
    warn!("Failed to send request to HTTP client: {}", error);

    ErrorMessage {
        code: ErrorCode::UnknownCode as i32,
        message: "Error during request!".to_string(),
        timestamp: Utc::now().timestamp_millis(),
        exchange_message: None,
    }
}
