use axum::body::Body;
use axum::http::{Request, StatusCode};
use infocar_monolith::config::AppConfig;
use infocar_monolith::http::routes::build_router;
use infocar_monolith::http::state::AppState;
use serde_json::{Value, json};
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_test_app(
    rest_url: String,
    soap_url: String,
    csv_url: String,
    retries: u32,
) -> axum::Router {
    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        provider_rest_url: rest_url,
        provider_soap_url: soap_url,
        provider_ai_url: csv_url.clone(),
        provider_csv_url: csv_url,
        payment_provider_url: "http://127.0.0.1:1".to_string(),
        provider_order: vec!["rest".to_string(), "soap".to_string(), "csv".to_string()],
        provider_timeout_secs: 2,
        provider_retries: retries,
        provider_backoff_ms: 5,
        log_file_path: "/tmp/infocar_resilience_test.log".to_string(),
    };
    build_router(AppState::from_config(config))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_http_fail_fast_chain_rest_to_soap_to_csv() {
    let rest_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/ABC1234/debts"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{corrupt_json"))
        .expect(1) // Fail-fast: apenas 1 chamada mesmo com retries = 2
        .mount(&rest_server)
        .await;

    let soap_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/soap"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<response><broken>"))
        .expect(1) // Fail-fast: apenas 1 chamada
        .mount(&soap_server)
        .await;

    let csv_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/debts/ABC1234"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "tipo,valor,vencimento\nIPVA,1500.00,2024-01-10\nMULTA,300.50,2024-02-15\n",
        ))
        .expect(1)
        .mount(&csv_server)
        .await;

    let app = build_test_app(rest_server.uri(), soap_server.uri(), csv_server.uri(), 2);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"ABC1234"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["placa"], "ABC1234");
    assert_eq!(body["resumo"]["total_atualizado"], "2355.93");

    let metrics_resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let metrics = body_json(metrics_resp).await;
    assert_eq!(metrics["metrics"]["provider_retries_total"], 0);
    assert_eq!(metrics["metrics"]["provider_fallbacks_total"], 2);
}

#[tokio::test]
async fn test_http_retry_on_500_then_fallback_to_soap() {
    let rest_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/LIC2024/debts"))
        .respond_with(ResponseTemplate::new(500))
        .expect(3) // 1 inicial + 2 retries
        .mount(&rest_server)
        .await;

    let soap_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/soap"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
    <plate>LIC2024</plate>
    <debts>
        <debt>
            <category>LICENCIAMENTO</category>
            <value>150.00</value>
            <expiration>2024-04-30</expiration>
        </debt>
    </debts>
</response>"#,
        ))
        .expect(1)
        .mount(&soap_server)
        .await;

    let csv_server = MockServer::start().await;
    let app = build_test_app(rest_server.uri(), soap_server.uri(), csv_server.uri(), 2);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/vehicles/debts")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"placa":"LIC2024"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["resumo"]["total_atualizado"], "154.95");
    assert_eq!(
        body["pagamentos"]["opcoes"][0]["pix"]["total_com_desconto"],
        json!("147.20")
    );
}
