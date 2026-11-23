use crate::application::plate_masker::PlateMasker;
use crate::domain::debt::contracts::VehicleDebtProvider;
use crate::domain::debt::entities::ProviderDebtResponse;
use crate::domain::debt::errors::ProviderError;
use crate::infrastructure::observability::logger::StructuredLogger;
use crate::infrastructure::observability::metrics::SimpleMetricsRegistry;
use async_trait::async_trait;
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[async_trait]
pub trait Sleeper: Send + Sync {
    async fn sleep(&self, ms: u64);
}

#[derive(Debug, Default, Clone)]
pub struct TokioSleeper;

#[async_trait]
impl Sleeper for TokioSleeper {
    async fn sleep(&self, ms: u64) {
        if ms > 0 {
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
    }
}

#[derive(Clone)]
pub struct ProviderExecutor {
    max_retries: u32,
    initial_backoff_ms: u64,
    metrics: Arc<SimpleMetricsRegistry>,
    logger: Arc<StructuredLogger>,
    sleeper: Arc<dyn Sleeper>,
}

impl ProviderExecutor {
    pub fn new(
        max_retries: u32,
        initial_backoff_ms: u64,
        metrics: Arc<SimpleMetricsRegistry>,
        logger: Arc<StructuredLogger>,
        sleeper: Arc<dyn Sleeper>,
    ) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            metrics,
            logger,
            sleeper,
        }
    }

    pub async fn execute(
        &self,
        provider_alias: &str,
        provider: &dyn VehicleDebtProvider,
        plate: &str,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, ProviderError> {
        let total_attempts = self.max_retries + 1;
        let masked_plate = PlateMasker::mask(plate);

        for attempt in 1..=total_attempts {
            self.metrics.increment("provider_requests_total");

            let mut req_fields = Map::new();
            req_fields.insert("provider".to_string(), Value::String(provider_alias.to_string()));
            req_fields.insert("operation".to_string(), Value::String("get_debts".to_string()));
            req_fields.insert("plate".to_string(), Value::String(masked_plate.clone()));
            req_fields.insert("attempt".to_string(), Value::from(attempt));
            self.logger
                .info("provider.request", "provider.request", request_id, req_fields);

            let start = Instant::now();
            match provider.get_debts(plate, request_id).await {
                Ok(response) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let mut ok_fields = Map::new();
                    ok_fields.insert(
                        "provider".to_string(),
                        Value::String(provider_alias.to_string()),
                    );
                    ok_fields.insert("status".to_string(), Value::String("success".to_string()));
                    ok_fields.insert("duration_ms".to_string(), Value::from(duration_ms));
                    ok_fields.insert(
                        "debts_count".to_string(),
                        Value::from(response.debts.len()),
                    );
                    self.logger.info(
                        "provider.response",
                        "provider.response",
                        request_id,
                        ok_fields,
                    );
                    return Ok(response);
                }
                Err(err) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    self.metrics.increment("provider_failures_total");

                    let is_last_attempt = attempt == total_attempts;
                    let is_fatal_for_provider = !err.is_retriable() || is_last_attempt;

                    let mut fail_fields = Map::new();
                    fail_fields.insert(
                        "provider".to_string(),
                        Value::String(provider_alias.to_string()),
                    );
                    fail_fields.insert("attempt".to_string(), Value::from(attempt));
                    fail_fields.insert("error".to_string(), Value::String(err.to_string()));
                    fail_fields.insert("duration_ms".to_string(), Value::from(duration_ms));
                    fail_fields.insert("plate".to_string(), Value::String(masked_plate.clone()));
                    fail_fields.insert(
                        "is_fatal_for_provider".to_string(),
                        Value::Bool(is_fatal_for_provider),
                    );
                    self.logger.warning(
                        "vehicle_provider_failed",
                        "External vehicle debt provider failed",
                        request_id,
                        fail_fields,
                    );

                    // Fail-fast para InvalidResponse / UnexpectedStatus, ou se esgotou tentativas
                    if !err.is_retriable() || is_last_attempt {
                        return Err(err);
                    }

                    // Retry com backoff linear: attempt * initial_backoff_ms
                    let backoff_ms = (attempt as u64) * self.initial_backoff_ms;
                    let mut retry_fields = Map::new();
                    retry_fields.insert(
                        "provider".to_string(),
                        Value::String(provider_alias.to_string()),
                    );
                    retry_fields.insert("attempt".to_string(), Value::from(attempt + 1));
                    retry_fields.insert("max_attempts".to_string(), Value::from(total_attempts));
                    retry_fields.insert("reason".to_string(), Value::String(err.to_string()));
                    retry_fields.insert("backoff_ms".to_string(), Value::from(backoff_ms));
                    self.logger.warning(
                        "provider.retry",
                        "provider.retry",
                        request_id,
                        retry_fields,
                    );

                    self.metrics.increment("provider_retries_total");
                    self.sleeper.sleep(backoff_ms).await;
                }
            }
        }

        unreachable!("Loop always returns on last attempt or success")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct RecordingSleeper {
        sleeps: Mutex<Vec<u64>>,
    }

    impl RecordingSleeper {
        fn new() -> Self {
            Self {
                sleeps: Mutex::new(Vec::new()),
            }
        }

        fn recorded(&self) -> Vec<u64> {
            self.sleeps.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl Sleeper for RecordingSleeper {
        async fn sleep(&self, ms: u64) {
            self.sleeps.lock().unwrap().push(ms);
        }
    }

    struct FlakyProvider {
        failures_before_success: u32,
        calls: AtomicU32,
        fatal_error: bool,
    }

    #[async_trait]
    impl VehicleDebtProvider for FlakyProvider {
        fn name(&self) -> &'static str {
            "rest"
        }

        async fn get_debts(
            &self,
            plate: &str,
            _request_id: Option<&str>,
        ) -> Result<ProviderDebtResponse, ProviderError> {
            let call_num = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if self.fatal_error {
                return Err(ProviderError::InvalidResponse("malformed json".to_string()));
            }
            if call_num <= self.failures_before_success {
                return Err(ProviderError::Unavailable("HTTP 500".to_string()));
            }
            Ok(ProviderDebtResponse::new(plate, vec![], "rest"))
        }
    }

    #[tokio::test]
    async fn test_executor_retries_linear_backoff_and_succeeds() {
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let sleeper = Arc::new(RecordingSleeper::new());
        let executor = ProviderExecutor::new(2, 100, metrics.clone(), logger, sleeper.clone());

        let provider = FlakyProvider {
            failures_before_success: 2,
            calls: AtomicU32::new(0),
            fatal_error: false,
        };

        let result = executor
            .execute("rest", &provider, "ABC1234", Some("req-1"))
            .await;

        assert!(result.is_ok());
        assert_eq!(provider.calls.load(Ordering::SeqCst), 3);
        assert_eq!(sleeper.recorded(), vec![100, 200]);
        assert_eq!(metrics.get("provider_requests_total"), 3);
        assert_eq!(metrics.get("provider_failures_total"), 2);
        assert_eq!(metrics.get("provider_retries_total"), 2);
    }

    #[tokio::test]
    async fn test_executor_fail_fast_on_invalid_response() {
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let sleeper = Arc::new(RecordingSleeper::new());
        let executor = ProviderExecutor::new(2, 100, metrics.clone(), logger, sleeper.clone());

        let provider = FlakyProvider {
            failures_before_success: 0,
            calls: AtomicU32::new(0),
            fatal_error: true,
        };

        let result = executor
            .execute("rest", &provider, "ABC1234", Some("req-2"))
            .await;

        assert!(matches!(result, Err(ProviderError::InvalidResponse(_))));
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        assert!(sleeper.recorded().is_empty());
        assert_eq!(metrics.get("provider_requests_total"), 1);
        assert_eq!(metrics.get("provider_failures_total"), 1);
        assert_eq!(metrics.get("provider_retries_total"), 0);
    }
}
