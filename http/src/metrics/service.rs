use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::{GenericMetricLayer, Handle, PrometheusMetricLayer};

pub struct MetricsService {
    handler: PrometheusHandle,
}

impl MetricsService {
    pub fn new<'a>() -> (
        GenericMetricLayer<'a, PrometheusHandle, Handle>,
        MetricsService,
    ) {
        let (prometheus_layer, handler): (
            GenericMetricLayer<PrometheusHandle, Handle>,
            PrometheusHandle,
        ) = PrometheusMetricLayer::pair();

        (prometheus_layer, Self { handler })
    }

    pub fn get(&self) -> String {
        self.handler.render()
    }
}
