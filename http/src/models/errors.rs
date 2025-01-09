use serde::Serialize;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Debug)]
pub struct HttpError {
    pub message: String,
    pub code: ErrorCode,
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} with code {}", self.message, self.code)
    }
}

impl Error for HttpError {}

#[derive(Serialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorCode {
    Internal,
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    Unavailable,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Internal => write!(f, "INTERNAL"),
            ErrorCode::NotFound => write!(f, "NOT_FOUND"),
            ErrorCode::Unavailable => write!(f, "UNAVAILABLE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::errors::{ErrorCode, HttpError};

    #[test]
    fn serialize_should_return_not_found_json() {
        let error = HttpError {
            message: String::from("Cannot find market!"),
            code: ErrorCode::NotFound,
        };

        let json = serde_json::to_string(&error);

        let expected = r#"{"message":"Cannot find market!","code":"NOT_FOUND"}"#;

        assert_eq!(json.ok(), Some(expected.to_string()));
    }

    #[test]
    fn display_should_return_correct_not_found() {
        let error = HttpError {
            message: String::from("Cannot find market"),
            code: ErrorCode::NotFound,
        };

        let expected = "Cannot find market with code NOT_FOUND";

        assert_eq!(error.to_string(), expected.to_string());
    }

    #[test]
    fn display_should_return_correct_errors() {
        assert_eq!(ErrorCode::NotFound.to_string(), "NOT_FOUND".to_string());
        assert_eq!(
            ErrorCode::Unavailable.to_string(),
            "UNAVAILABLE".to_string()
        );
        assert_eq!(ErrorCode::Internal.to_string(), "INTERNAL".to_string());
    }
}
