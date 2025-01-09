use serde::Deserialize;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Debug)]
pub struct ExchangeResponse<T> {
    pub result: T,
}

#[derive(Deserialize, Debug)]
pub struct ExchangeError {
    pub error: String,
}

impl Display for ExchangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Exchange error: {}", self.error)
    }
}

impl Error for ExchangeError {}
