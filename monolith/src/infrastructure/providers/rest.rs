use crate::domain::debt::contracts::VehicleDebtProvider;
use crate::domain::debt::entities::{Debt, ProviderDebtResponse};
use crate::domain::debt::errors::ProviderError;
use crate::domain::debt::money::Money;
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct RestVehicleDebtProvider {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl RestVehicleDebtProvider {
    pub fn new(client: Client, base_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout,
        }
    }

    fn normalize_amount(val: &Value) -> Result<Money, ProviderError> {
        let raw_decimal = match val {
            Value::String(s) => s.trim().to_string(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    format!("{}.00", i)
                } else if let Some(u) = n.as_u64() {
                    format!("{}.00", u)
                } else if let Some(f) = n.as_f64() {
                    format!("{:.2}", f)
                } else {
                    return Err(ProviderError::InvalidResponse(
                        "Invalid numeric amount in REST provider".to_string(),
                    ));
                }
            }
            _ => {
                return Err(ProviderError::InvalidResponse(
                    "Debt amount must be string or number".to_string(),
                ));
            }
        };

        Money::from_decimal(&raw_decimal).map_err(|e| ProviderError::InvalidResponse(e.to_string()))
    }
}

#[async_trait]
impl VehicleDebtProvider for RestVehicleDebtProvider {
    fn name(&self) -> &'static str {
        "rest"
    }

    async fn get_debts(
        &self,
        plate: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError> {
        let url = format!("{}/api/v1/vehicles/{}/debts", self.base_url, plate);
        let mut req = self.client.get(&url).timeout(self.timeout);

        if let Some(rid) = request_id {
            req = req.header("X-Request-ID", rid);
        }

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("REST request failed: {}", e)))?;

        let status = response.status();
        if status.is_server_error() {
            return Err(ProviderError::Unavailable(format!(
                "REST provider returned HTTP {}",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(ProviderError::UnexpectedStatus(format!(
                "REST provider returned unexpected HTTP {}",
                status.as_u16()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("Failed reading REST body: {}", e)))?;

        let json: Value = serde_json::from_str(&body)
            .map_err(|e| ProviderError::InvalidResponse(format!("Malformed JSON: {}", e)))?;

        let debts_array = json
            .get("debts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::InvalidResponse("Missing or invalid 'debts' array".to_string())
            })?;

        let mut debts = Vec::with_capacity(debts_array.len());
        for item in debts_array {
            let obj = item.as_object().ok_or_else(|| {
                ProviderError::InvalidResponse("Debt item must be a JSON object".to_string())
            })?;

            let raw_type = obj
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    ProviderError::InvalidResponse("Missing or invalid debt 'type'".to_string())
                })?;

            let amount_val = obj.get("amount").ok_or_else(|| {
                ProviderError::InvalidResponse("Missing debt 'amount'".to_string())
            })?;
            let amount = Self::normalize_amount(amount_val)?;

            let due_date_str = obj
                .get("due_date")
                .and_then(|v| v.as_str())
                .map(|s| s.trim())
                .ok_or_else(|| {
                    ProviderError::InvalidResponse("Missing or invalid 'due_date'".to_string())
                })?;
            let due_date = NaiveDate::parse_from_str(due_date_str, "%Y-%m-%d").map_err(|e| {
                ProviderError::InvalidResponse(format!("Invalid due_date '{}': {}", due_date_str, e))
            })?;

            debts.push(Debt::new(raw_type, amount, due_date));
        }

        Ok(ProviderDebtResponse::new(plate, debts, "rest"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_rest_provider_success_and_request_id_propagation() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/vehicles/ABC1234/debts"))
            .and(header("X-Request-ID", "req-rest-1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "plate": "ABC1234",
                "debts": [
                    { "type": "IPVA", "amount": 1500.0, "due_date": "2024-01-10" },
                    { "type": "MULTA", "amount": "300.50", "due_date": "2024-02-15" }
                ]
            })))
            .mount(&server)
            .await;

        let provider = RestVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let res = provider.get_debts("ABC1234", Some("req-rest-1")).await.unwrap();

        assert_eq!(res.provider, "rest");
        assert_eq!(res.debts.len(), 2);
        assert_eq!(res.debts[0].amount.to_decimal(), "1500.00");
        assert_eq!(res.debts[1].amount.to_decimal(), "300.50");
    }

    #[tokio::test]
    async fn test_rest_provider_zero_debts() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/vehicles/ZERO000/debts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "plate": "ZERO000",
                "debts": []
            })))
            .mount(&server)
            .await;

        let provider = RestVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let res = provider.get_debts("ZERO000", None).await.unwrap();
        assert!(res.debts.is_empty());
    }

    #[tokio::test]
    async fn test_rest_provider_invalid_json_returns_invalid_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{not-json"))
            .mount(&server)
            .await;

        let provider = RestVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let err = provider.get_debts("ABC1234", None).await.unwrap_err();
        assert!(matches!(err, ProviderError::InvalidResponse(_)));
    }

    #[tokio::test]
    async fn test_rest_provider_http_500_and_timeout_return_unavailable() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/vehicles/ERR5000/debts"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/vehicles/TMO0000/debts"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(300)))
            .mount(&server)
            .await;

        let provider = RestVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_millis(80));
        let err_500 = provider.get_debts("ERR5000", None).await.unwrap_err();
        assert!(matches!(err_500, ProviderError::Unavailable(_)));

        let err_tmo = provider.get_debts("TMO0000", None).await.unwrap_err();
        assert!(matches!(err_tmo, ProviderError::Unavailable(_)));
    }
}
