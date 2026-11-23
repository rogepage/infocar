use axum::body::Body;
use axum::http::{Request, StatusCode};
use infocar_monolith::config::AppConfig;
use infocar_monolith::http::middleware::rate_limit::RateLimiter;
use infocar_monolith::http::routes::build_router;
use infocar_monolith::http::state::AppState;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config(rest_url: String, soap_url: String, csv_url: String, pay_url: String) -> AppConfig {
    AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        provider_rest_url: rest_url,
        provider_soap_url: soap_url,
        provider_ai_url: csv_url.clone(),
        provider_csv_url: csv_url,
        payment_provider_url: pay_url,
        provider_order: vec!["rest".to_string(), "soap".to_string(), "csv".to_string()],
        provider_timeout_secs: 2,
        provider_retries: 0,
        provider_backoff_ms: 10,
        log_file_path: "/tmp/infocar_phase4_test.log".to_string(),
    }
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_http_health_security_headers_and_request_id() {
    let server = MockServer::start().await;
    let state = AppState::from_config(test_config(
        server.uri(),
        server.uri(),
        server.uri(),
        server.uri(),
    ));
    let app = build_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .header("X-Request-ID", "custom-req-42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers().get("x-request-id").unwrap(), "custom-req-42");
    assert_eq!(resp.headers().get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "SAMEORIGIN");
    assert_eq!(resp.headers().get("x-xss-protection").unwrap(), "1; mode=block");
    assert_eq!(
        resp.headers().get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );

    let json = body_json(resp).await;
    assert_eq!(json, json!({ "status": "ok", "service": "monolith" }));
}

#[tokio::test]
async fn test_post_vehicle_debts_200_and_validation_errors() {
    let rest_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/ABC1234/debts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "plate": "ABC1234",
            "debts": [
                { "type": "IPVA", "amount": "1500.00", "due_date": "2024-01-10" },
                { "type": "MULTA", "amount": "300.50", "due_date": "2024-02-15" }
            ]
        })))
        .mount(&rest_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/DPV0001/debts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "plate": "DPV0001",
            "debts": [
                { "type": "SEGURO_DPVAT", "amount": "50.00", "due_date": "2024-01-10" }
            ]
        })))
        .mount(&rest_server)
        .await;

    let dead_server = MockServer::start().await;
    let state = AppState::from_config(test_config(
        rest_server.uri(),
        dead_server.uri(),
        dead_server.uri(),
        dead_server.uri(),
    ));
    let app = build_router(state);

    // 1. 200 OK
    let resp_ok = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"abc1234"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_ok.status(), StatusCode::OK);
    let json_ok = body_json(resp_ok).await;
    assert_eq!(json_ok["placa"], "ABC1234");
    assert_eq!(json_ok["resumo"]["total_atualizado"], "2355.93");
    assert_eq!(json_ok["pagamentos"]["opcoes"][0]["pix"]["total_com_desconto"], "2238.13");

    // 2. 400 unknown_field
    let resp_unknown = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"ABC1234","foo":"bar"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_unknown.status(), StatusCode::BAD_REQUEST);
    let json_unknown = body_json(resp_unknown).await;
    assert_eq!(json_unknown["error"], "unknown_field");
    assert_eq!(json_unknown["unrecognized_fields"], json!(["foo"]));

    // 3. 400 invalid_plate
    let resp_invalid = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"INVALID"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_invalid.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(resp_invalid).await, json!({ "error": "invalid_plate" }));

    // 4. 422 unknown_debt_type
    let resp_422 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"DPV0001","provider":"rest"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_422.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        body_json(resp_422).await,
        json!({ "error": "unknown_debt_type", "type": "SEGURO_DPVAT" })
    );

    // 5. 503 all_providers_unavailable
    let resp_503 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"ERR9999"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_503.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body_json(resp_503).await,
        json!({ "error": "all_providers_unavailable" })
    );

    // 6. Metrics check
    let resp_metrics = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_metrics.status(), StatusCode::OK);
    let metrics_json = body_json(resp_metrics).await;
    assert_eq!(metrics_json["metrics"]["vehicle_debt_requests_total"], 5);
    assert_eq!(metrics_json["metrics"]["vehicle_debt_success_total"], 1);
    assert_eq!(metrics_json["metrics"]["vehicle_debt_error_total"], 4);
}

#[tokio::test]
async fn test_health_integrations_cache_hit_miss_and_no_cache() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"ok"})))
        .mount(&server)
        .await;

    let state = AppState::from_config(test_config(
        server.uri(),
        server.uri(),
        server.uri(),
        server.uri(),
    ));
    let app = build_router(state);

    // 1st call -> MISS
    let r1 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health/integrations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::OK);
    assert_eq!(r1.headers().get("x-cache").unwrap(), "MISS");

    // 2nd call -> HIT
    let r2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health/integrations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r2.status(), StatusCode::OK);
    assert_eq!(r2.headers().get("x-cache").unwrap(), "HIT");

    // 3rd call with Cache-Control: no-cache -> MISS
    let r3 = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health/integrations")
                .header("Cache-Control", "no-cache")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r3.status(), StatusCode::OK);
    assert_eq!(r3.headers().get("x-cache").unwrap(), "MISS");
}

#[tokio::test]
async fn test_rate_limit_returns_429_when_exceeded() {
    let server = MockServer::start().await;
    let mut state = AppState::from_config(test_config(
        server.uri(),
        server.uri(),
        server.uri(),
        server.uri(),
    ));
    state.rate_limiter = Arc::new(RateLimiter::new(2, Duration::from_secs(60)));
    let app = build_router(state);

    for _ in 0..2 {
        let r = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/vehicles/debts")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"placa":"INVALID"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }

    let r_limited = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"INVALID"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r_limited.status(), StatusCode::TOO_MANY_REQUESTS);
}
