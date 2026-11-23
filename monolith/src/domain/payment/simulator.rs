use crate::domain::debt::debt_type::DebtType;
use crate::domain::debt::entities::CalculatedVehicleDebts;
use crate::domain::debt::errors::DomainError;
use crate::domain::debt::money::Money;
use crate::domain::payment::credit_card_calculator::CreditCardCalculator;
use crate::domain::payment::dto::{PaymentOption, PaymentSimulationResult};
use crate::domain::payment::pix_calculator::PixCalculator;

#[derive(Clone, Default)]
pub struct PaymentSimulator {
    pix_calculator: PixCalculator,
    credit_card_calculator: CreditCardCalculator,
}

impl PaymentSimulator {
    pub fn new(
        pix_calculator: PixCalculator,
        credit_card_calculator: CreditCardCalculator,
    ) -> Self {
        Self {
            pix_calculator,
            credit_card_calculator,
        }
    }

    pub fn simulate(
        &self,
        calculated: &CalculatedVehicleDebts,
    ) -> Result<PaymentSimulationResult, DomainError> {
        if calculated.debts.is_empty() || !calculated.total_updated.is_positive() {
            return Ok(PaymentSimulationResult {
                options: Vec::new(),
            });
        }

        let mut options = Vec::new();

        // 1. Opção TOTAL
        options.push(self.build_option("TOTAL", calculated.total_updated)?);

        // 2. Opções SOMENTE_<TIPO> na ordem de primeira aparição dos tipos
        let mut seen_order: Vec<DebtType> = Vec::new();
        let mut sums_by_type: Vec<(DebtType, Money)> = Vec::new();

        for debt in &calculated.debts {
            if let Some(pos) = seen_order.iter().position(|&t| t == debt.debt_type) {
                sums_by_type[pos].1 = sums_by_type[pos].1.add(debt.updated_amount);
            } else {
                seen_order.push(debt.debt_type);
                sums_by_type.push((debt.debt_type, debt.updated_amount));
            }
        }

        for (debt_type, category_total) in sums_by_type {
            if category_total.is_positive() {
                let option_type = format!("SOMENTE_{}", debt_type.as_str());
                options.push(self.build_option(&option_type, category_total)?);
            }
        }

        Ok(PaymentSimulationResult { options })
    }

    fn build_option(
        &self,
        option_type: &str,
        base_amount: Money,
    ) -> Result<PaymentOption, DomainError> {
        let pix = self.pix_calculator.calculate(base_amount)?;
        let credit_card = self.credit_card_calculator.calculate(base_amount);

        Ok(PaymentOption {
            option_type: option_type.to_string(),
            base_amount,
            pix,
            credit_card,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::debt::calculation_service::DebtCalculationService;
    use crate::domain::debt::clock::FixedClock;
    use crate::domain::debt::entities::Debt;
    use crate::domain::debt::policies::DebtInterestPolicyRegistry;
    use chrono::NaiveDate;
    use std::sync::Arc;

    #[test]
    fn test_payment_simulator_generates_total_and_per_type_in_appearance_order() {
        let clock = Arc::new(FixedClock::default_reference());
        let calc_service =
            DebtCalculationService::new(clock, DebtInterestPolicyRegistry::with_defaults());
        let simulator = PaymentSimulator::default();

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

        let calculated = calc_service.calculate("ABC1234", &debts).unwrap();
        let simulation = simulator.simulate(&calculated).unwrap();

        assert_eq!(simulation.options.len(), 3);
        assert_eq!(simulation.options[0].option_type, "TOTAL");
        assert_eq!(simulation.options[0].base_amount.to_decimal(), "2355.93");
        assert_eq!(
            simulation.options[0].pix.total_with_discount.to_decimal(),
            "2238.13"
        );

        assert_eq!(simulation.options[1].option_type, "SOMENTE_IPVA");
        assert_eq!(simulation.options[1].base_amount.to_decimal(), "1800.00");

        assert_eq!(simulation.options[2].option_type, "SOMENTE_MULTA");
        assert_eq!(simulation.options[2].base_amount.to_decimal(), "555.93");
    }

    #[test]
    fn test_payment_simulator_returns_empty_for_zero_debts() {
        let clock = Arc::new(FixedClock::default_reference());
        let calc_service =
            DebtCalculationService::new(clock, DebtInterestPolicyRegistry::with_defaults());
        let simulator = PaymentSimulator::default();

        let calculated = calc_service.calculate("ZERO000", &[]).unwrap();
        let simulation = simulator.simulate(&calculated).unwrap();

        assert!(simulation.options.is_empty());
    }
}
