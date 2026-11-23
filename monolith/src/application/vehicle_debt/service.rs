use crate::application::plate_masker::PlateMasker;
use crate::application::vehicle_debt::executor::ProviderExecutor;
use crate::application::vehicle_debt::resolver::ProviderResolver;
use crate::domain::debt::entities::ProviderDebtResponse;
use crate::domain::debt::errors::DomainError;
use crate::infrastructure::observability::logger::StructuredLogger;
use crate::infrastructure::observability::metrics::SimpleMetricsRegistry;
use serde_json::{Map, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct VehicleDebtService {
    resolver: Arc<ProviderResolver>,
    executor: Arc<ProviderExecutor>,
    metrics: Arc<SimpleMetricsRegistry>,
    logger: Arc<StructuredLogger>,
}

impl VehicleDebtService {
    pub fn new(
        resolver: Arc<ProviderResolver>,
        executor: Arc<ProviderExecutor>,
        metrics: Arc<SimpleMetricsRegistry>,
        logger: Arc<StructuredLogger>,
    ) -> Self {
        Self {
            resolver,
            executor,
            metrics,
            logger,
        }
    }

    pub async fn get_debts(
        &self,
        plate: &str,
        preferred_provider: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<ProviderDebtResponse, DomainError> {
        let custom_order = preferred_provider.map(|p| vec![p.to_string()]);
        let providers = self.resolver.resolve_order(custom_order.as_deref());

        if providers.is_empty() {
            return Err(DomainError::AllProvidersUnavailable(
                "No vehicle debt providers configured".to_string(),
            ));
        }

        let masked_plate = PlateMasker::mask(plate);
        let mut tried_providers = Vec::new();
        let mut errors = Vec::new();

        for (idx, (alias, provider)) in providers.iter().enumerate() {
            tried_providers.push(alias.clone());

            match self
                .executor
                .execute(alias, provider.as_ref(), plate, request_id)
                .await
            {
                Ok(response) => {
                    return Ok(response);
                }
                Err(err) => {
                    let err_msg = err.to_string();
                    errors.push(format!("{}: {}", alias, err_msg));

                    if let Some((next_alias, _)) = providers.get(idx + 1) {
                        self.metrics.increment("provider_fallbacks_total");

                        let mut fb_fields = Map::new();
                        fb_fields.insert("from".to_string(), Value::String(alias.clone()));
                        fb_fields.insert("to".to_string(), Value::String(next_alias.clone()));
                        fb_fields.insert("reason".to_string(), Value::String(err_msg));
                        fb_fields.insert("plate".to_string(), Value::String(masked_plate.clone()));
                        self.logger.warning(
                            "provider.fallback",
                            "provider.fallback",
                            request_id,
                            fb_fields,
                        );
                    }
                }
            }
        }

        let mut err_fields = Map::new();
        err_fields.insert("plate".to_string(), Value::String(masked_plate));
        err_fields.insert(
            "tried_providers".to_string(),
            Value::Array(tried_providers.into_iter().map(Value::String).collect()),
        );
        err_fields.insert(
            "errors".to_string(),
            Value::Array(errors.iter().cloned().map(Value::String).collect()),
        );
        self.logger.error(
            "all_vehicle_providers_failed",
            "All vehicle debt providers failed",
            request_id,
            err_fields,
        );

        Err(DomainError::AllProvidersUnavailable(errors.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::vehicle_debt::executor::Sleeper;
    use crate::domain::debt::contracts::VehicleDebtProvider;
    use crate::domain::debt::entities::Debt;
    use crate::domain::debt::errors::ProviderError;
    use crate::domain::debt::money::Money;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct NoopSleeper;
    #[async_trait]
    impl Sleeper for NoopSleeper {
        async fn sleep(&self, _ms: u64) {}
    }

    struct StubProvider {
        name: &'static str,
        should_fail: bool,
        debts_count: usize,
        calls: AtomicU32,
    }

    impl StubProvider {
        fn ok(name: &'static str, debts_count: usize) -> Self {
            Self {
                name,
                should_fail: false,
                debts_count,
                calls: AtomicU32::new(0),
            }
        }

        fn failing(name: &'static str) -> Self {
            Self {
                name,
                should_fail: true,
                debts_count: 0,
                calls: AtomicU32::new(0),
            }
        }
    }

    #[async_trait]
    impl VehicleDebtProvider for StubProvider {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn get_debts(
            &self,
            plate: &str,
            _request_id: Option<&str>,
        ) -> Result<ProviderDebtResponse, ProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                return Err(ProviderError::Unavailable("500".to_string()));
            }
            let debts = (0..self.debts_count)
                .map(|_| {
                    Debt::new(
                        "IPVA",
                        Money::from_decimal("100.00").unwrap(),
                        NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                    )
                })
                .collect();
            Ok(ProviderDebtResponse::new(plate, debts, self.name))
        }
    }

    #[tokio::test]
    async fn test_first_success_wins_fallback_from_rest_to_soap() {
        let rest = Arc::new(StubProvider::failing("rest"));
        let soap = Arc::new(StubProvider::ok("soap", 1));
        let csv = Arc::new(StubProvider::ok("ai", 1));

        let mut map: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        map.insert("rest".to_string(), rest.clone());
        map.insert("soap".to_string(), soap.clone());
        map.insert("csv".to_string(), csv.clone());

        let resolver = Arc::new(ProviderResolver::new(
            map,
            vec!["rest".to_string(), "soap".to_string(), "csv".to_string()],
        ));
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let executor = Arc::new(ProviderExecutor::new(
            1,
            10,
            metrics.clone(),
            logger.clone(),
            Arc::new(NoopSleeper),
        ));
        let service = VehicleDebtService::new(resolver, executor, metrics.clone(), logger);

        let res = service.get_debts("ABC1234", None, Some("req-fb")).await.unwrap();
        assert_eq!(res.provider, "soap");
        assert_eq!(rest.calls.load(Ordering::SeqCst), 2);
        assert_eq!(soap.calls.load(Ordering::SeqCst), 1);
        assert_eq!(csv.calls.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.get("provider_fallbacks_total"), 1);
    }

    #[tokio::test]
    async fn test_zero_debts_does_not_trigger_fallback() {
        let rest = Arc::new(StubProvider::ok("rest", 0));
        let soap = Arc::new(StubProvider::ok("soap", 1));

        let mut map: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        map.insert("rest".to_string(), rest.clone());
        map.insert("soap".to_string(), soap.clone());

        let resolver = Arc::new(ProviderResolver::new(
            map,
            vec!["rest".to_string(), "soap".to_string()],
        ));
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let executor = Arc::new(ProviderExecutor::new(
            2,
            10,
            metrics.clone(),
            logger.clone(),
            Arc::new(NoopSleeper),
        ));
        let service = VehicleDebtService::new(resolver, executor, metrics.clone(), logger);

        let res = service.get_debts("ZERO000", None, Some("req-zero")).await.unwrap();
        assert_eq!(res.provider, "rest");
        assert!(res.debts.is_empty());
        assert_eq!(soap.calls.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.get("provider_fallbacks_total"), 0);
    }

    #[tokio::test]
    async fn test_all_providers_unavailable_returns_domain_error() {
        let rest = Arc::new(StubProvider::failing("rest"));
        let soap = Arc::new(StubProvider::failing("soap"));

        let mut map: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        map.insert("rest".to_string(), rest);
        map.insert("soap".to_string(), soap);

        let resolver = Arc::new(ProviderResolver::new(
            map,
            vec!["rest".to_string(), "soap".to_string()],
        ));
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let executor = Arc::new(ProviderExecutor::new(
            0,
            10,
            metrics,
            logger.clone(),
            Arc::new(NoopSleeper),
        ));
        let service = VehicleDebtService::new(resolver, executor, Arc::new(SimpleMetricsRegistry::new()), logger);

        let err = service.get_debts("ABC1234", None, None).await.unwrap_err();
        assert!(matches!(err, DomainError::AllProvidersUnavailable(_)));
    }
}
