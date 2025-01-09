use serde::Serialize;

const NAME: &str = "http";

pub trait HealthCheck: Send + Sync {
    fn check(&self) -> HealthcheckResult;
}

#[derive(Serialize, PartialEq, Debug)]
pub struct HealthcheckResult {
    pub service: String,
    pub enabled: bool,
}

pub struct HttpHealthCheck;

impl HealthCheck for HttpHealthCheck {
    fn check(&self) -> HealthcheckResult {
        HealthcheckResult {
            service: String::from(NAME),
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::checks::{HealthCheck, HealthcheckResult, HttpHealthCheck, NAME};

    #[test]
    fn http_check_should_return_true() {
        let check = HttpHealthCheck;

        let expected = HealthcheckResult {
            service: String::from(NAME),
            enabled: true,
        };

        assert_eq!(check.check(), expected);
    }
}
