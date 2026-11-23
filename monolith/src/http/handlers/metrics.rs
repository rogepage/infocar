use crate::http::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub async fn get_metrics(State(state): State<AppState>) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "metrics": state.metrics.get_all()
        })),
    )
        .into_response()
}
