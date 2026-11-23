use crate::domain::debt::entities::CalculatedVehicleDebts;
use crate::domain::payment::dto::PaymentSimulationResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VehicleDebtConsultationResult {
    pub calculated: CalculatedVehicleDebts,
    pub payment_simulation: PaymentSimulationResult,
    pub provider_used: String,
}
