use crate::healthcheck::service::HealthcheckService;
use crate::metrics::service::MetricsService;
use crate::utils::errors::not_found_handler;
use crate::{healthcheck, metrics};
use axum::routing::get;
use axum::Router;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::{GenericMetricLayer, Handle};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct HttpConfig {
    host: String,
    port: u16,
}

impl HttpConfig {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub fn base_router(healthcheck_service: HealthcheckService) -> Router<()> {
    let (prometheus_layer, metric_service): (
        GenericMetricLayer<PrometheusHandle, Handle>,
        MetricsService,
    ) = MetricsService::new();

    Router::new()
        .route(healthcheck::api::PATH, get(healthcheck::api::health_check))
        .with_state(Arc::new(healthcheck_service))
        .route(metrics::api::PATH, get(metrics::api::fetch))
        .with_state(Arc::new(metric_service))
        .layer(prometheus_layer)
        .fallback(not_found_handler)
}
