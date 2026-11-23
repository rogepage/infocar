use crate::http::state::AppState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use regex::Regex;
use serde_json::{Map, Value};
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

fn request_id_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-zA-Z0-9\-_]{1,64}$").unwrap())
}

pub async fn request_id_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let incoming_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| request_id_regex().is_match(s));

    let req_id = incoming_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    req.extensions_mut().insert(RequestId(req_id.clone()));

    let path = req.uri().path().trim_start_matches('/').to_string();
    let method = req.method().as_str().to_string();

    if !path.starts_with("api/health") && path != "up" {
        let mut fields = Map::new();
        fields.insert("method".to_string(), Value::String(method));
        fields.insert("path".to_string(), Value::String(path.clone()));
        state
            .logger
            .info("request.received", "request.received", Some(&req_id), fields);
    }

    if path.contains("vehicles/debts") {
        state.metrics.increment("vehicle_debt_requests_total");
    }

    let mut response = next.run(req).await;
    if let Ok(header_val) = HeaderValue::from_str(&req_id) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), header_val);
    }

    response
}
