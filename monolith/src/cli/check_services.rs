use crate::config::AppConfig;
use reqwest::Client;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ServiceCheckResult {
    pub name: String,
    pub url: String,
    pub status: &'static str,
    pub http_code: u16,
    pub latency_ms: u128,
    pub message: String,
    pub healthy: bool,
}

async fn probe_service(client: &Client, name: &str, url: String) -> ServiceCheckResult {
    let start = Instant::now();
    match client.get(&url).timeout(Duration::from_secs(3)).send().await {
        Ok(resp) => {
            let latency_ms = start.elapsed().as_millis();
            let http_code = resp.status().as_u16();
            let healthy = resp.status().is_success();
            let body = resp.text().await.unwrap_or_default();
            let snippet: String = body.trim().chars().take(80).collect();

            ServiceCheckResult {
                name: name.to_string(),
                url,
                status: if healthy { "ONLINE" } else { "FAIL" },
                http_code,
                latency_ms,
                message: snippet,
                healthy,
            }
        }
        Err(err) => {
            let latency_ms = start.elapsed().as_millis();
            ServiceCheckResult {
                name: name.to_string(),
                url,
                status: "UNREACHABLE",
                http_code: 0,
                latency_ms,
                message: err.to_string().chars().take(80).collect(),
                healthy: false,
            }
        }
    }
}

pub async fn check_all_services(config: &AppConfig) -> Vec<ServiceCheckResult> {
    let client = Client::new();
    let targets = [
        (
            "Provider REST",
            format!("{}/api/health", config.provider_rest_url.trim_end_matches('/')),
        ),
        (
            "Provider SOAP",
            format!("{}/health", config.provider_soap_url.trim_end_matches('/')),
        ),
        (
            "Provider CSV/AI",
            format!("{}/api/health", config.provider_csv_url.trim_end_matches('/')),
        ),
        (
            "Payment Provider",
            format!("{}/health", config.payment_provider_url.trim_end_matches('/')),
        ),
    ];

    let mut results = Vec::with_capacity(targets.len());
    for (name, url) in targets {
        results.push(probe_service(&client, name, url).await);
    }
    results
}

pub async fn run_check_services(config: &AppConfig) -> bool {
    let results = check_all_services(config).await;

    println!(
        "{:<18} | {:<40} | {:<11} | {:<9} | {:<10} | {}",
        "Serviço", "URL Alvo", "Status", "HTTP Code", "Latência", "Resposta"
    );
    println!("{}", "-".repeat(120));

    let mut all_healthy = true;
    for r in &results {
        if !r.healthy {
            all_healthy = false;
        }
        println!(
            "{:<18} | {:<40} | {:<11} | {:<9} | {:<10} | {}",
            r.name,
            r.url,
            r.status,
            r.http_code,
            format!("{}ms", r.latency_ms),
            r.message
        );
    }

    all_healthy
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_check_services_all_online_and_degraded() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/health"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"status":"ok"}"#))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"status":"ok"}"#))
            .mount(&server)
            .await;

        let mut cfg = AppConfig::from_env();
        cfg.provider_rest_url = server.uri();
        cfg.provider_soap_url = server.uri();
        cfg.provider_csv_url = server.uri();
        cfg.payment_provider_url = server.uri();

        assert!(run_check_services(&cfg).await);

        let down_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&down_server)
            .await;
        cfg.payment_provider_url = down_server.uri();

        assert!(!run_check_services(&cfg).await);
    }
}
