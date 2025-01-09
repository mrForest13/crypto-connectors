use http::healthcheck::checks::{HealthCheck, HealthcheckResult};
use http::healthcheck::service::HealthcheckService;
use protocol::client::NatsClient;
use std::sync::Arc;

const NAME: &str = "nats";

struct NatsHealthCheck {
    nats_client: Arc<NatsClient>,
}

impl NatsHealthCheck {
    pub fn new(nats_client: Arc<NatsClient>) -> NatsHealthCheck {
        NatsHealthCheck { nats_client }
    }

    fn check(&self) -> bool {
        self.nats_client.is_healthy()
    }
}

impl HealthCheck for NatsHealthCheck {
    fn check(&self) -> HealthcheckResult {
        HealthcheckResult {
            service: String::from(NAME),
            enabled: self.check(),
        }
    }
}

pub fn nats_healthcheck(nats_client: Arc<NatsClient>) -> HealthcheckService {
    let mut healthcheck: HealthcheckService = HealthcheckService::default();
    healthcheck.add(Box::new(NatsHealthCheck::new(nats_client)));
    healthcheck
}
