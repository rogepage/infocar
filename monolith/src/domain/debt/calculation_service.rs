use crate::domain::debt::clock::Clock;
use crate::domain::debt::debt_type::DebtType;
use crate::domain::debt::entities::{CalculatedDebt, CalculatedVehicleDebts, Debt};
use crate::domain::debt::errors::DomainError;
use crate::domain::debt::money::Money;
use crate::domain::debt::policies::DebtInterestPolicyRegistry;
use std::sync::Arc;

#[derive(Clone)]
pub struct DebtCalculationService {
    clock: Arc<dyn Clock>,
    registry: DebtInterestPolicyRegistry,
}

impl DebtCalculationService {
    pub fn new(clock: Arc<dyn Clock>, registry: DebtInterestPolicyRegistry) -> Self {
        Self { clock, registry }
    }

    pub fn calculate(
        &self,
        plate: &str,
        debts: &[Debt],
    ) -> Result<CalculatedVehicleDebts, DomainError> {
        let ref_date = self.clock.today_date();
        let mut calculated_debts = Vec::with_capacity(debts.len());
        let mut total_original = Money::zero();
        let mut total_updated = Money::zero();

        for debt in debts {
            let normalized_raw = debt.raw_type.trim().to_uppercase();
            let debt_type = DebtType::try_from_normalized(&normalized_raw)
                .ok_or_else(|| DomainError::UnknownDebtType(normalized_raw.clone()))?;

            let policy = self
                .registry
                .get(debt_type)
                .ok_or_else(|| DomainError::UnknownDebtType(normalized_raw))?;

            let diff_days = (ref_date - debt.due_date).num_days();
            let days_overdue = diff_days.max(0);

            let interest_amount = policy.calculate_interest(debt.amount, days_overdue)?;
            let updated_amount = debt.amount.add(interest_amount);

            total_original = total_original.add(debt.amount);
            total_updated = total_updated.add(updated_amount);

            calculated_debts.push(CalculatedDebt {
                debt_type,
                original_amount: debt.amount,
                updated_amount,
                interest_amount,
                due_date: debt.due_date,
                days_overdue,
            });
        }

        Ok(CalculatedVehicleDebts {
            plate: plate.to_string(),
            debts: calculated_debts,
            total_original,
            total_updated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::debt::clock::FixedClock;
    use chrono::NaiveDate;

    #[test]
    fn test_calculate_abc1234_canonical_vector() {
        let clock = Arc::new(FixedClock::default_reference());
        let registry = DebtInterestPolicyRegistry::with_defaults();
        let service = DebtCalculationService::new(clock, registry);

        let debts = vec![
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
        ];

        let result = service.calculate("ABC1234", &debts).unwrap();

        assert_eq!(result.plate, "ABC1234");
        assert_eq!(result.debts.len(), 2);

        assert_eq!(result.debts[0].debt_type, DebtType::Ipva);
        assert_eq!(result.debts[0].days_overdue, 121);
        assert_eq!(result.debts[0].original_amount.to_decimal(), "1500.00");
        assert_eq!(result.debts[0].updated_amount.to_decimal(), "1800.00");

        assert_eq!(result.debts[1].debt_type, DebtType::Multa);
        assert_eq!(result.debts[1].days_overdue, 85);
        assert_eq!(result.debts[1].original_amount.to_decimal(), "300.50");
        assert_eq!(result.debts[1].updated_amount.to_decimal(), "555.93");

        assert_eq!(result.total_original.to_decimal(), "1800.50");
        assert_eq!(result.total_updated.to_decimal(), "2355.93");
    }

    #[test]
    fn test_calculate_rejects_unknown_debt_type() {
        let clock = Arc::new(FixedClock::default_reference());
        let registry = DebtInterestPolicyRegistry::with_defaults();
        let service = DebtCalculationService::new(clock, registry);

        let debts = vec![Debt::new(
            "SEGURO_DPVAT",
            Money::from_decimal("50.00").unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        )];

        let err = service.calculate("ABC1234", &debts).unwrap_err();
        assert_eq!(
            err,
            DomainError::UnknownDebtType("SEGURO_DPVAT".to_string())
        );
    }

    #[test]
    fn test_calculate_future_due_date_has_zero_days_overdue() {
        let clock = Arc::new(FixedClock::default_reference());
        let registry = DebtInterestPolicyRegistry::with_defaults();
        let service = DebtCalculationService::new(clock, registry);

        let debts = vec![Debt::new(
            "IPVA",
            Money::from_decimal("1000.00").unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )];

        let result = service.calculate("ABC1234", &debts).unwrap();
        assert_eq!(result.debts[0].days_overdue, 0);
        assert_eq!(result.debts[0].updated_amount.to_decimal(), "1000.00");
    }
}
