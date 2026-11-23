use crate::http::handlers::health::{get_health, get_health_integrations};
use crate::http::handlers::metrics::get_metrics;
use crate::http::handlers::vehicle_debts::post_vehicle_debts;
use crate::http::handlers::web_ui::{get_web_ui, post_web_ui};
use crate::http::middleware::rate_limit::rate_limit_middleware;
use crate::http::middleware::request_id::request_id_middleware;
use crate::http::middleware::security_headers::security_headers_middleware;
use crate::http::state::AppState;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

pub fn build_router(state: AppState) -> Router {
    let debts_route = Router::new()
        .route("/api/v1/vehicles/debts", post(post_vehicle_debts))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    Router::new()
        .route("/", get(get_web_ui).post(post_web_ui))
        .merge(debts_route)
        .route("/api/health", get(get_health))
        .route("/api/health/integrations", get(get_health_integrations))
        .route("/api/metrics", get(get_metrics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_id_middleware,
        ))
        .layer(middleware::from_fn(security_headers_middleware))
        .with_state(state)
}
