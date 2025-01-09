pub mod api;
pub mod checks;
pub mod service;

#[cfg(test)]
mod tests {
    use crate::healthcheck::checks::{HealthCheck, HealthcheckResult};

    pub struct DbHealthCheck {}

    impl HealthCheck for DbHealthCheck {
        fn check(&self) -> HealthcheckResult {
            HealthcheckResult {
                service: "db".to_string(),
                enabled: false,
            }
        }
    }
}
