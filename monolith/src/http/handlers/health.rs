use crate::http::state::{AppState, CachedHealthEntry};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};
use std::time::{Duration, Instant};

const HEALTH_CACHE_TTL: Duration = Duration::from_secs(5);

pub async fn get_health() -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "monolith"
        })),
    )
        .into_response()
}

async fn check_service(client: &reqwest::Client, url: &str) -> (bool, Value) {
    let req = client.get(url).timeout(Duration::from_secs(2));
    match req.send().await {
        Ok(resp) => {
            let code = resp.status().as_u16();
            let is_ok = resp.status().is_success();
            let data = resp
                .json::<Value>()
                .await
                .unwrap_or_else(|_| json!({}));

            let entry = json!({
                "status": if is_ok { "ok" } else { "error" },
                "url": url,
                "http_code": code,
                "data": data,
            });
            (is_ok, entry)
        }
        Err(err) => {
            let entry = json!({
                "status": "unreachable",
                "url": url,
                "http_code": 0,
                "error": err.to_string(),
            });
            (false, entry)
        }
    }
}

pub async fn get_health_integrations(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let bypass_cache = headers
        .get("Cache-Control")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase().contains("no-cache"))
        .unwrap_or(false);

    if !bypass_cache {
        let guard = state.health_cache.read().await;
        if let Some(entry) = guard.as_ref()
            && entry.cached_at.elapsed() < HEALTH_CACHE_TTL
        {
            let mut resp = (entry.status_code, Json(entry.body.clone())).into_response();
            resp.headers_mut().insert(
                HeaderName::from_static("x-cache"),
                HeaderValue::from_static("HIT"),
            );
            return resp;
        }
    }

    let rest_url = format!(
        "{}/api/health",
        state.config.provider_rest_url.trim_end_matches('/')
    );
    let soap_url = format!(
        "{}/health",
        state.config.provider_soap_url.trim_end_matches('/')
    );
    let payment_url = format!(
        "{}/health",
        state.config.payment_provider_url.trim_end_matches('/')
    );

    let ((rest_ok, rest_val), (soap_ok, soap_val), (pay_ok, pay_val)) = tokio::join!(
        check_service(&state.http_client, &rest_url),
        check_service(&state.http_client, &soap_url),
        check_service(&state.http_client, &payment_url),
    );

    let all_ok = rest_ok && soap_ok && pay_ok;
    let status_code = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let mut services = Map::new();
    services.insert("provider-rest".to_string(), rest_val);
    services.insert("provider-soap".to_string(), soap_val);
    services.insert("payment-provider".to_string(), pay_val);

    let body = json!({
        "status": if all_ok { "ok" } else { "degraded" },
        "service": "monolith",
        "services": services,
    });

    {
        let mut guard = state.health_cache.write().await;
        *guard = Some(CachedHealthEntry {
            cached_at: Instant::now(),
            status_code,
            body: body.clone(),
        });
    }

    let mut resp = (status_code, Json(body)).into_response();
    resp.headers_mut().insert(
        HeaderName::from_static("x-cache"),
        HeaderValue::from_static("MISS"),
    );
    resp
}
