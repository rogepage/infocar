use crate::domain::debt::debt_type::DebtType;
use crate::domain::debt::money::Money;
use chrono::NaiveDate;

/// Entidade canônica representando um débito bruto normalizado por um provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Debt {
    pub raw_type: String,
    pub amount: Money,
    pub due_date: NaiveDate,
}

impl Debt {
    pub fn new(raw_type: impl Into<String>, amount: Money, due_date: NaiveDate) -> Self {
        Self {
            raw_type: raw_type.into(),
            amount,
            due_date,
        }
    }
}

/// Envelope canônico retornado por qualquer implementação de `VehicleDebtProvider`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDebtResponse {
    pub plate: String,
    pub debts: Vec<Debt>,
    pub provider: String,
}

impl ProviderDebtResponse {
    pub fn new(plate: impl Into<String>, debts: Vec<Debt>, provider: impl Into<String>) -> Self {
        Self {
            plate: plate.into(),
            debts,
            provider: provider.into(),
        }
    }
}

/// Resultado do cálculo de juros de um débito individual.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatedDebt {
    pub debt_type: DebtType,
    pub original_amount: Money,
    pub updated_amount: Money,
    pub interest_amount: Money,
    pub due_date: NaiveDate,
    pub days_overdue: i64,
}

/// Resultado consolidado dos débitos calculados de um veículo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatedVehicleDebts {
    pub plate: String,
    pub debts: Vec<CalculatedDebt>,
    pub total_original: Money,
    pub total_updated: Money,
}
