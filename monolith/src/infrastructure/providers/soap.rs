use crate::domain::debt::contracts::VehicleDebtProvider;
use crate::domain::debt::entities::{Debt, ProviderDebtResponse};
use crate::domain::debt::errors::ProviderError;
use crate::domain::debt::money::Money;
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use roxmltree::Document;
use std::time::Duration;

#[derive(Clone)]
pub struct SoapVehicleDebtProvider {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl SoapVehicleDebtProvider {
    pub fn new(client: Client, base_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout,
        }
    }

    fn build_request_xml(plate: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<request>\n    <plate>{}</plate>\n</request>",
            plate
        )
    }

    fn parse_response_xml(plate: &str, body: &str) -> Result<ProviderDebtResponse, ProviderError> {
        if body.trim().is_empty() {
            return Err(ProviderError::InvalidResponse(
                "Empty SOAP XML response".to_string(),
            ));
        }

        let doc = Document::parse(body)
            .map_err(|e| ProviderError::InvalidResponse(format!("Malformed XML: {}", e)))?;

        let root = doc.root_element();
        if root.tag_name().name() != "response" {
            return Err(ProviderError::InvalidResponse(format!(
                "Expected <response> root element, got <{}>",
                root.tag_name().name()
            )));
        }

        let debts_node = root
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "debts")
            .ok_or_else(|| {
                ProviderError::InvalidResponse("Missing <debts> element in SOAP XML".to_string())
            })?;

        let mut debts = Vec::new();
        for debt_node in debts_node
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "debt")
        {
            let category = Self::extract_child_text(&debt_node, "category")?;
            let value_str = Self::extract_child_text(&debt_node, "value")?;
            let expiration_str = Self::extract_child_text(&debt_node, "expiration")?;

            let amount = Money::from_decimal(&value_str)
                .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;
            let due_date = NaiveDate::parse_from_str(&expiration_str, "%Y-%m-%d").map_err(|e| {
                ProviderError::InvalidResponse(format!(
                    "Invalid SOAP expiration date '{}': {}",
                    expiration_str, e
                ))
            })?;

            debts.push(Debt::new(category, amount, due_date));
        }

        Ok(ProviderDebtResponse::new(plate, debts, "soap"))
    }

    fn extract_child_text(node: &roxmltree::Node<'_, '_>, tag: &str) -> Result<String, ProviderError> {
        node.children()
            .find(|n| n.is_element() && n.tag_name().name() == tag)
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                ProviderError::InvalidResponse(format!("Missing or empty <{}> in <debt>", tag))
            })
    }
}

#[async_trait]
impl VehicleDebtProvider for SoapVehicleDebtProvider {
    fn name(&self) -> &'static str {
        "soap"
    }

    async fn get_debts(
        &self,
        plate: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError> {
        let url = format!("{}/soap", self.base_url);
        let xml_payload = Self::build_request_xml(plate);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/xml")
            .body(xml_payload)
            .timeout(self.timeout);

        if let Some(rid) = request_id {
            req = req.header("X-Request-ID", rid);
        }

        let response = req
            .send()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("SOAP request failed: {}", e)))?;

        let status = response.status();
        if status.is_server_error() {
            return Err(ProviderError::Unavailable(format!(
                "SOAP provider returned HTTP {}",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(ProviderError::UnexpectedStatus(format!(
                "SOAP provider returned unexpected HTTP {}",
                status.as_u16()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| ProviderError::Unavailable(format!("Failed reading SOAP body: {}", e)))?;

        Self::parse_response_xml(plate, &body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_soap_provider_success_xml() {
        let server = MockServer::start().await;
        let xml_body = r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
    <plate>ABC1234</plate>
    <debts>
        <debt>
            <category>IPVA</category>
            <value>1500.00</value>
            <expiration>2024-01-10</expiration>
        </debt>
    </debts>
</response>"#;

        Mock::given(method("POST"))
            .and(path("/soap"))
            .and(header("Content-Type", "application/xml"))
            .and(header("X-Request-ID", "req-soap-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml_body))
            .mount(&server)
            .await;

        let provider = SoapVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let res = provider.get_debts("ABC1234", Some("req-soap-1")).await.unwrap();

        assert_eq!(res.provider, "soap");
        assert_eq!(res.debts.len(), 1);
        assert_eq!(res.debts[0].raw_type, "IPVA");
        assert_eq!(res.debts[0].amount.to_decimal(), "1500.00");
    }

    #[tokio::test]
    async fn test_soap_provider_self_closing_empty_debts() {
        let server = MockServer::start().await;
        let xml_body = r#"<?xml version="1.0" encoding="UTF-8"?><response><plate>ZERO000</plate><debts/></response>"#;

        Mock::given(method("POST"))
            .and(path("/soap"))
            .respond_with(ResponseTemplate::new(200).set_body_string(xml_body))
            .mount(&server)
            .await;

        let provider = SoapVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let res = provider.get_debts("ZERO000", None).await.unwrap();
        assert!(res.debts.is_empty());
    }

    #[tokio::test]
    async fn test_soap_provider_malformed_xml_returns_invalid_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/soap"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<response><unclosed>"))
            .mount(&server)
            .await;

        let provider = SoapVehicleDebtProvider::new(Client::new(), server.uri(), Duration::from_secs(2));
        let err = provider.get_debts("ABC1234", None).await.unwrap_err();
        assert!(matches!(err, ProviderError::InvalidResponse(_)));
    }
}
