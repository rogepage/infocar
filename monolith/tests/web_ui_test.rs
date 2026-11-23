use axum::body::Body;
use axum::http::{Request, StatusCode};
use infocar_monolith::config::AppConfig;
use infocar_monolith::http::routes::build_router;
use infocar_monolith::http::state::AppState;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn build_ui_app(rest_url: String) -> axum::Router {
    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        provider_rest_url: rest_url,
        provider_soap_url: "http://127.0.0.1:1".to_string(),
        provider_ai_url: "http://127.0.0.1:1".to_string(),
        provider_csv_url: "http://127.0.0.1:1".to_string(),
        payment_provider_url: "http://127.0.0.1:1".to_string(),
        provider_order: vec!["rest".to_string()],
        provider_timeout_secs: 2,
        provider_retries: 0,
        provider_backoff_ms: 10,
        log_file_path: "/tmp/infocar_webui_test.log".to_string(),
    };
    build_router(AppState::from_config(config))
}

async fn body_text(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn test_web_ui_initial_form_and_lookup_states() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/ABC1234/debts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "plate": "ABC1234",
            "debts": [
                { "type": "IPVA", "amount": "1500.00", "due_date": "2024-01-10" }
            ]
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/vehicles/ZER0000/debts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "plate": "ZER0000",
            "debts": []
        })))
        .mount(&server)
        .await;

    let app = build_ui_app(server.uri());

    // 1. GET /
    let r_get = app
        .clone()
        .oneshot(Request::builder().method("GET").uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(r_get.status(), StatusCode::OK);
    let html_get = body_text(r_get).await;
    assert!(html_get.contains("Consulta de Débitos Veiculares"));
    assert!(html_get.contains("Consultar débitos"));

    // 2. POST / com débitos
    let r_ok = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("placa=ABC1234"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r_ok.status(), StatusCode::OK);
    let html_ok = body_text(r_ok).await;
    assert!(html_ok.contains("R$ 1.500,00"));
    assert!(html_ok.contains("10/01/2024"));
    assert!(html_ok.contains("Pagamento Total dos Débitos"));
    assert!(html_ok.contains("PIX à Vista (5% OFF)"));

    // 3. POST / sem débitos
    let r_zero = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("placa=ZER0000"))
                .unwrap(),
        )
        .await
        .unwrap();
    let html_zero = body_text(r_zero).await;
    assert!(html_zero.contains("Nenhum débito encontrado."));

    // 4. POST / placa inválida
    let r_inv = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("placa=INVALID"))
                .unwrap(),
        )
        .await
        .unwrap();
    let html_inv = body_text(r_inv).await;
    assert!(html_inv.contains("Placa inválida. Verifique o formato informado."));
}
