use crate::application::vehicle_debt::result::VehicleDebtConsultationResult;
use crate::application::vehicle_debt::service::VehicleDebtService;
use crate::domain::debt::calculation_service::DebtCalculationService;
use crate::domain::debt::errors::DomainError;
use crate::domain::payment::simulator::PaymentSimulator;
use std::sync::Arc;

#[derive(Clone)]
pub struct GetVehicleDebtsUseCase {
    vehicle_debt_service: Arc<VehicleDebtService>,
    calculation_service: Arc<DebtCalculationService>,
    payment_simulator: Arc<PaymentSimulator>,
}

impl GetVehicleDebtsUseCase {
    pub fn new(
        vehicle_debt_service: Arc<VehicleDebtService>,
        calculation_service: Arc<DebtCalculationService>,
        payment_simulator: Arc<PaymentSimulator>,
    ) -> Self {
        Self {
            vehicle_debt_service,
            calculation_service,
            payment_simulator,
        }
    }

    pub async fn execute(
        &self,
        plate: &str,
        preferred_provider: Option<&str>,
        request_id: Option<&str>,
    ) -> Result<VehicleDebtConsultationResult, DomainError> {
        let normalized_plate = plate.trim().to_uppercase();
        let provider_response = self
            .vehicle_debt_service
            .get_debts(&normalized_plate, preferred_provider, request_id)
            .await?;

        let calculated = self
            .calculation_service
            .calculate(&normalized_plate, &provider_response.debts)?;

        let payment_simulation = self.payment_simulator.simulate(&calculated)?;

        Ok(VehicleDebtConsultationResult {
            calculated,
            payment_simulation,
            provider_used: provider_response.provider,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::vehicle_debt::executor::{ProviderExecutor, Sleeper};
    use crate::application::vehicle_debt::resolver::ProviderResolver;
    use crate::domain::debt::clock::FixedClock;
    use crate::domain::debt::contracts::VehicleDebtProvider;
    use crate::domain::debt::entities::{Debt, ProviderDebtResponse};
    use crate::domain::debt::errors::ProviderError;
    use crate::domain::debt::money::Money;
    use crate::domain::debt::policies::DebtInterestPolicyRegistry;
    use crate::infrastructure::observability::logger::StructuredLogger;
    use crate::infrastructure::observability::metrics::SimpleMetricsRegistry;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use std::collections::HashMap;

    struct NoopSleeper;
    #[async_trait]
    impl Sleeper for NoopSleeper {
        async fn sleep(&self, _ms: u64) {}
    }

    struct MockProvider;
    #[async_trait]
    impl VehicleDebtProvider for MockProvider {
        fn name(&self) -> &'static str {
            "rest"
        }

        async fn get_debts(
            &self,
            plate: &str,
            _request_id: Option<&str>,
        ) -> Result<ProviderDebtResponse, ProviderError> {
            Ok(ProviderDebtResponse::new(
                plate,
                vec![
                    Debt::new(
                        "IPVA",
                        Money::from_decimal("1500.00").unwrap(),
                        NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                    ),
                    Debt::new(
                        "MULTA",
                        Money::from_decimal("300.50").unwrap(),
                        NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                    ),
                ],
                "rest",
            ))
        }
    }

    #[tokio::test]
    async fn test_get_vehicle_debts_use_case_end_to_end() {
        let mut map: HashMap<String, Arc<dyn VehicleDebtProvider>> = HashMap::new();
        map.insert("rest".to_string(), Arc::new(MockProvider));
        let resolver = Arc::new(ProviderResolver::new(map, vec!["rest".to_string()]));
        let metrics = Arc::new(SimpleMetricsRegistry::new());
        let logger = Arc::new(StructuredLogger::noop());
        let executor = Arc::new(ProviderExecutor::new(
            2,
            100,
            metrics.clone(),
            logger.clone(),
            Arc::new(NoopSleeper),
        ));
        let debt_service = Arc::new(VehicleDebtService::new(resolver, executor, metrics, logger));

        let calc_service = Arc::new(DebtCalculationService::new(
            Arc::new(FixedClock::default_reference()),
            DebtInterestPolicyRegistry::with_defaults(),
        ));
        let simulator = Arc::new(PaymentSimulator::default());
        let use_case = GetVehicleDebtsUseCase::new(debt_service, calc_service, simulator);

        let result = use_case
            .execute(" abc1234 ", None, Some("req-uc"))
            .await
            .unwrap();

        assert_eq!(result.provider_used, "rest");
        assert_eq!(result.calculated.plate, "ABC1234");
        assert_eq!(result.calculated.total_updated.to_decimal(), "2355.93");
        assert_eq!(result.payment_simulation.options.len(), 3);
        assert_eq!(
            result.payment_simulation.options[0]
                .pix
                .total_with_discount
                .to_decimal(),
            "2238.13"
        );
    }
}
