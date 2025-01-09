use crate::healthcheck::checks::{HealthCheck, HealthcheckResult, HttpHealthCheck};

pub struct HealthcheckService {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl Default for HealthcheckService {
    fn default() -> Self {
        let service: HttpHealthCheck = HttpHealthCheck {};
        let check: Box<dyn HealthCheck> = Box::new(service);

        HealthcheckService {
            checks: vec![check],
        }
    }
}

impl HealthcheckService {
    pub fn add(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }
}

impl HealthcheckService {
    pub fn health_check(&self) -> Vec<HealthcheckResult> {
        self.checks.iter().map(|checker| checker.check()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::checks::HealthcheckResult;
    use crate::healthcheck::service::HealthcheckService;
    use crate::healthcheck::tests::DbHealthCheck;

    #[test]
    fn health_check_should_return_true_for_http_and_false_for_db() {
        let mut service: HealthcheckService = HealthcheckService::default();

        service.add(Box::new(DbHealthCheck {}));

        let expected = vec![
            HealthcheckResult {
                service: String::from("http"),
                enabled: true,
            },
            HealthcheckResult {
                service: String::from("db"),
                enabled: false,
            },
        ];

        assert_eq!(service.health_check(), expected);
    }
}
