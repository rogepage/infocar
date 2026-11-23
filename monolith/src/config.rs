use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub provider_rest_url: String,
    pub provider_soap_url: String,
    pub provider_ai_url: String,
    pub provider_csv_url: String,
    pub payment_provider_url: String,
    pub provider_order: Vec<String>,
    pub provider_timeout_secs: u64,
    pub provider_retries: u32,
    pub provider_backoff_ms: u64,
    pub log_file_path: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8000);

        let provider_rest_url = env::var("PROVIDER_REST_URL")
            .unwrap_or_else(|_| "http://provider-rest:8000".to_string());
        let provider_soap_url = env::var("PROVIDER_SOAP_URL")
            .unwrap_or_else(|_| "http://provider-soap:8000".to_string());
        let provider_csv_url = env::var("PROVIDER_CSV_URL")
            .or_else(|_| env::var("PROVIDER_AI_URL"))
            .unwrap_or_else(|_| "http://provider-csv:8000".to_string());
        let provider_ai_url = env::var("PROVIDER_AI_URL")
            .unwrap_or_else(|_| provider_csv_url.clone());
        let payment_provider_url = env::var("PAYMENT_PROVIDER_URL")
            .unwrap_or_else(|_| "http://payment-provider:8000".to_string());

        let raw_order = env::var("PROVIDER_ORDER")
            .unwrap_or_else(|_| "rest,soap,csv".to_string());
        let provider_order = Self::parse_provider_order(&raw_order);

        let provider_timeout_secs = env::var("PROVIDER_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(2);

        let provider_retries = env::var("PROVIDER_RETRIES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(2);

        let provider_backoff_ms = env::var("PROVIDER_BACKOFF_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(100);

        let log_file_path = env::var("LOG_FILE_PATH")
            .unwrap_or_else(|_| "storage/logs/laravel.log".to_string());

        Self {
            host,
            port,
            provider_rest_url,
            provider_soap_url,
            provider_ai_url,
            provider_csv_url,
            payment_provider_url,
            provider_order,
            provider_timeout_secs,
            provider_retries,
            provider_backoff_ms,
            log_file_path,
        }
    }

    pub fn parse_provider_order(raw: &str) -> Vec<String> {
        let parsed: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if parsed.is_empty() {
            vec!["rest".to_string(), "soap".to_string(), "csv".to_string()]
        } else {
            parsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_provider_order_normalizes_and_handles_empty() {
        assert_eq!(
            AppConfig::parse_provider_order(" REST , soap , CSV "),
            vec!["rest", "soap", "csv"]
        );
        assert_eq!(
            AppConfig::parse_provider_order("   ,  "),
            vec!["rest", "soap", "csv"]
        );
    }
}
