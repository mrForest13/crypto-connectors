use crate::healthcheck::checks::HealthcheckResult;
use crate::healthcheck::service::HealthcheckService;
use crate::models::response::{ErrorResponse, OkResponse};
use crate::utils::errors;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;

pub const PATH: &str = "/admin/healthcheck";

pub async fn health_check(State(service): State<Arc<HealthcheckService>>) -> impl IntoResponse {
    let checks: Vec<HealthcheckResult> = service.health_check();

    if checks.iter().any(|check| !check.enabled) {
        Err(ErrorResponse::one(
            errors::unavailable(),
            StatusCode::SERVICE_UNAVAILABLE,
        ))
    } else {
        Ok(OkResponse::new(checks, StatusCode::OK))
    }
}

#[cfg(test)]
mod tests {
    use crate::healthcheck::api::{health_check, PATH};
    use crate::healthcheck::service::HealthcheckService;
    use crate::healthcheck::tests::DbHealthCheck;
    use crate::utils::errors::not_found_handler;
    use axum::{routing::get, Router};
    use axum_test::TestServer;
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn health_check_return_success() {
        let service: HealthcheckService = HealthcheckService::default();
        let app: Router = Router::new()
            .route(PATH, get(health_check))
            .with_state(Arc::new(service));

        let server = TestServer::new(app).unwrap();
        let response = server.get("/admin/healthcheck").await;

        let expected = json!({
            "data": [
                {
                    "service": "http",
                    "enabled": true
                }
            ]
        });

        response.assert_status_ok();
        response.assert_json_contains(&expected)
    }

    #[tokio::test]
    async fn health_check_return_failure() {
        let mut service: HealthcheckService = HealthcheckService::default();
        service.add(Box::new(DbHealthCheck {}));
        let app: Router = Router::new()
            .route("/health", get(health_check))
            .with_state(Arc::new(service));

        let server = TestServer::new(app).unwrap();
        let response = server.get("/health").await;

        let expected = json!({
            "data": [
                {
                    "message": "One of the services is unavailable!",
                    "code": "UNAVAILABLE",
                }
            ]
        });

        response.assert_status_service_unavailable();
        response.assert_json_contains(&expected)
    }

    async fn example() {}

    #[tokio::test]
    async fn health_check_return_page_not_found() {
        let app: Router = Router::new()
            .route("/example", get(example))
            .fallback(not_found_handler);

        let server = TestServer::new(app).unwrap();
        let response = server.get("/health").await;

        let expected = json!({
            "data": [
                {
                    "code": "NOT_FOUND",
                    "message": "Page not found",
                }
            ]
        });

        response.assert_status_not_found();
        response.assert_json_contains(&expected)
    }
}
