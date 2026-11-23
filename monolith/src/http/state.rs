use crate::application::vehicle_debt::executor::{ProviderExecutor, TokioSleeper};
use crate::application::vehicle_debt::resolver::ProviderResolver;
use crate::application::vehicle_debt::service::VehicleDebtService;
use crate::application::vehicle_debt::use_case::GetVehicleDebtsUseCase;
use crate::config::AppConfig;
use crate::domain::debt::calculation_service::DebtCalculationService;
use crate::domain::debt::clock::FixedClock;
use crate::domain::debt::contracts::VehicleDebtProvider;
use crate::domain::debt::policies::DebtInterestPolicyRegistry;
use crate::domain::payment::simulator::PaymentSimulator;
use crate::http::middleware::rate_limit::RateLimiter;
use crate::infrastructure::observability::logger::StructuredLogger;
use crate::infrastructure::observability::metrics::SimpleMetricsRegistry;
use crate::infrastructure::providers::ai_csv::AiVehicleDebtProvider;
use crate::infrastructure::providers::rest::RestVehicleDebtProvider;
use crate::infrastructure::providers::soap::SoapVehicleDebtProvider;
use axum::http::StatusCode;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CachedHealthEntry {
    pub cached_at: Instant,
    pub status_code: StatusCode,
    pub body: Value,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub use_case: Arc<GetVehicleDebtsUseCase>,
    pub metrics: Arc<SimpleMetricsRegistry>,
    pub logger: Arc<StructuredLogger>,
    pub http_client: Client,
    pub health_cache: Arc<RwLock<Option<CachedHealthEntry>>>,
    pub rate_limiter: Arc<RateLimiter>,
}

impl AppState {
    pub fn from_config(config: AppConfig) -> Self {
        let http_client = Client::new();
        let timeout = Duration::from_secs(config.provider_timeout_secs);
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::new(
            Some(PathBuf::from(&config.log_file_path)),
            true,
        ));

        let rest = Arc::new(RestVehicleDebtProvider::new(
            http_client.clone(),
            config.provider_rest_url.clone(),
            timeout,
        ));
        let soap = Arc::new(SoapVehicleDebtProvider::new(
            http_client.clone(),
            config.provider_soap_url.clone(),
            timeout,
        ));
        let ai_csv = Arc::new(AiVehicleDebtProvider::new(
            http_client.clone(),
            config.provider_csv_url.clone(),
            timeout,
            logger.clone(),
        ));

        let mut providers: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        providers.insert("rest".to_string(), rest);
        providers.insert("soap".to_string(), soap);
        providers.insert("ai".to_string(), ai_csv.clone());
        providers.insert("csv".to_string(), ai_csv);

        let resolver = Arc::new(ProviderResolver::new(
            providers,
            config.provider_order.clone(),
        ));
        let executor = Arc::new(ProviderExecutor::new(
            config.provider_retries,
            config.provider_backoff_ms,
            metrics.clone(),
            logger.clone(),
            Arc::new(TokioSleeper),
        ));
        let debt_service = Arc::new(VehicleDebtService::new(
            resolver,
            executor,
            metrics.clone(),
            logger.clone(),
        ));

        let calc_service = Arc::new(DebtCalculationService::new(
            Arc::new(FixedClock::default_reference()),
            DebtInterestPolicyRegistry::with_defaults(),
        ));
        let simulator = Arc::new(PaymentSimulator::default());
        let use_case = Arc::new(GetVehicleDebtsUseCase::new(
            debt_service,
            calc_service,
            simulator,
        ));

        Self {
            config,
            use_case,
            metrics,
            logger,
            http_client,
            health_cache: Arc::new(RwLock::new(None)),
            rate_limiter: Arc::new(RateLimiter::new(60, Duration::from_secs(60))),
        }
    }
}
