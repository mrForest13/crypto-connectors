use crate::metrics::service::MetricsService;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

pub const PATH: &str = "/admin/metrics";

pub async fn fetch(State(service): State<Arc<MetricsService>>) -> impl IntoResponse {
    Response::new(service.get())
}

#[cfg(test)]
mod tests {
    use crate::metrics::api::{fetch, PATH};
    use crate::metrics::service::MetricsService;
    use axum::{routing::get, Router};
    use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
    use axum_prometheus::{GenericMetricLayer, Handle};
    use axum_test::TestServer;
    use std::sync::Arc;

    #[tokio::test]
    async fn metrics_check_return_success() {
        let (layer, service): (GenericMetricLayer<PrometheusHandle, Handle>, MetricsService) =
            MetricsService::new();
        let app = Router::new()
            .route(PATH, get(fetch))
            .with_state(Arc::new(service))
            .layer(layer);

        let server = TestServer::new(app).unwrap();
        let response = server.get("/admin/metrics").await;

        response.assert_status_ok();
    }
}
