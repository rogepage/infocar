use crate::domain::debt::contracts::VehicleDebtProvider;
use crate::domain::debt::entities::{Debt, ProviderDebtResponse};
use crate::domain::debt::errors::ProviderError;
use crate::domain::debt::money::Money;
use crate::infrastructure::observability::logger::StructuredLogger;
use async_trait::async_trait;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use reqwest::Client;
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AiVehicleDebtProvider {
    client: Client,
    base_url: String,
    timeout: Duration,
    logger: Arc<StructuredLogger>,
}

impl AiVehicleDebtProvider {
    const SUPPORTED_TYPES: [&'static str; 3] = ["IPVA", "MULTA", "LICENCIAMENTO"];
    const HEADER_TOKENS: [&'static str; 4] = ["tipo", "type", "categoria", "category"];
    const INFORMATIVE_TOKENS: [&'static str; 7] = [
        "débito", "debito", "encontrado", "aqui", "veículo", "veiculo", "detran",
    ];

    pub fn new(
        client: Client,
        base_url: impl Into<String>,
        timeout: Duration,
        logger: Arc<StructuredLogger>,
    ) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout,
            logger,
        }
    }

    fn parse_csv_body(
        &self,
        plate: &str,
        body: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError> {
        if body.trim().is_empty() {
            return Ok(ProviderDebtResponse::new(plate, Vec::new(), "ai"));
        }

        let mut debts = Vec::new();
        let mut corrupt_lines_found: usize = 0;
        let mut valid_data_lines_found: usize = 0;

        for raw_line in body.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with("```") {
                continue;
            }

            let semicolon_count = trimmed.matches(';').count();
            let comma_count = trimmed.matches(',').count();
            let delimiter = if semicolon_count > comma_count {
                b';'
            } else {
                b','
            };

            let mut reader = ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .flexible(true)
                .from_reader(trimmed.as_bytes());

            let record = match reader.records().next() {
                Some(Ok(rec)) => rec,
                Some(Err(_)) => {
                    corrupt_lines_found += 1;
                    continue;
                }
                None => continue,
            };

            let cols: Vec<String> = record
                .iter()
                .map(|c| c.trim().trim_matches('"').trim_matches('\'').trim().to_string())
                .collect();

            if let Some(first) = cols.first()
                && Self::HEADER_TOKENS.contains(&first.to_lowercase().as_str())
            {
                continue;
            }

            if cols.len() < 3 {
                let lower = trimmed.to_lowercase();
                if Self::INFORMATIVE_TOKENS.iter().any(|t| lower.contains(t)) {
                    continue;
                }
                corrupt_lines_found += 1;
                continue;
            }

            let raw_type = cols[0].to_uppercase();
            let raw_amount = cols[1].replace(',', ".");
            let raw_date = &cols[2];

            let amount = match Money::from_decimal(&raw_amount) {
                Ok(m) => m,
                Err(_) => {
                    corrupt_lines_found += 1;
                    continue;
                }
            };

            let due_date = match NaiveDate::parse_from_str(raw_date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    corrupt_lines_found += 1;
                    continue;
                }
            };

            valid_data_lines_found += 1;

            if !Self::SUPPORTED_TYPES.contains(&raw_type.as_str()) {
                let mut fields = Map::new();
                fields.insert("raw_type".to_string(), Value::String(raw_type));
                fields.insert("raw_line".to_string(), Value::String(trimmed.to_string()));
                self.logger.warning(
                    "ai_provider.ignored_debt_type",
                    "Ignored unsupported debt type from AI provider",
                    request_id,
                    fields,
                );
                continue;
            }

            debts.push(Debt::new(raw_type, amount, due_date));
        }

        if corrupt_lines_found > 0 && valid_data_lines_found == 0 && debts.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "AI CSV response is completely malformed".to_string(),
            ));
        }

        Ok(ProviderDebtResponse::new(plate, debts, "ai"))
    }
}

#[async_trait]
impl VehicleDebtProvider for AiVehicleDebtProvider {
    fn name(&self) -> &'static str {
        "ai"
    }

    async fn get_debts(
        &self,
        plate: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError> {
        let url = format!("{}/api/v1/debts/{}", self.base_url, plate);
        let mut req = self
            .client
            .get(&url)
            .header("Accept", "text/plain, text/csv")
            .timeout(self.timeout);

        if let Some(rid) = request_id {
            req = req.header("X-Request-ID", rid);
        }

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("AI/CSV request failed: {}", e)))?;

        let status = response.status();
        if status.is_server_error() {
            return Err(ProviderError::Unavailable(format!(
                "AI/CSV provider returned HTTP {}",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(ProviderError::UnexpectedStatus(format!(
                "AI/CSV provider returned unexpected HTTP {}",
                status.as_u16()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("Failed reading AI/CSV body: {}", e)))?;

        self.parse_csv_body(plate, &body, request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_ai_csv_provider_parses_markdown_semicolon_and_filters_dpvat() {
        let server = MockServer::start().await;
        let csv_body = r#"Aqui estão os débitos encontrados para o veículo:
```csv
tipo;valor;vencimento
IPVA;"1500,00";2024-01-10
DPVAT;50.00;2024-01-20
MULTA,300.50,2024-02-15
```"#;

        Mock::given(method("GET"))
            .and(path("/api/v1/debts/ABC1234"))
            .respond_with(ResponseTemplate::new(200).set_body_string(csv_body))
            .mount(&server)
            .await;

        let logger = Arc::new(StructuredLogger::noop());
        let provider = AiVehicleDebtProvider::new(
            Client::new(),
            server.uri(),
            Duration::from_secs(2),
            logger,
        );

        let res = provider.get_debts("ABC1234", Some("req-csv-1")).await.unwrap();
        assert_eq!(res.provider, "ai");
        assert_eq!(res.debts.len(), 2);
        assert_eq!(res.debts[0].raw_type, "IPVA");
        assert_eq!(res.debts[0].amount.to_decimal(), "1500.00");
        assert_eq!(res.debts[1].raw_type, "MULTA");
        assert_eq!(res.debts[1].amount.to_decimal(), "300.50");
    }

    #[tokio::test]
    async fn test_ai_csv_provider_rejects_completely_corrupt_csv() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/debts/CORR000"))
            .respond_with(ResponseTemplate::new(200).set_body_string("bad;line;not_a_date\nfoo;bar"))
            .mount(&server)
            .await;

        let logger = Arc::new(StructuredLogger::noop());
        let provider = AiVehicleDebtProvider::new(
            Client::new(),
            server.uri(),
            Duration::from_secs(2),
            logger,
        );

        let err = provider.get_debts("CORR000", None).await.unwrap_err();
        assert!(matches!(err, ProviderError::InvalidResponse(_)));
    }
}
