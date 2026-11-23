use crate::domain::debt::debt_type::DebtType;
use crate::domain::debt::errors::DomainError;
use crate::domain::debt::money::Money;
use crate::domain::debt::rounding::HalfUpRounder;
use std::collections::HashMap;
use std::sync::Arc;

pub trait DebtInterestPolicy: Send + Sync {
    fn supports(&self) -> DebtType;
    fn calculate_interest(&self, original_amount: Money, days_overdue: i64) -> Result<Money, DomainError>;
}

/// Política de Juros para IPVA: 0,33% ao dia limitado ao teto de 20% antes do arredondamento HALF_UP.
pub struct IpvaInterestPolicy;

impl DebtInterestPolicy for IpvaInterestPolicy {
    fn supports(&self) -> DebtType {
        DebtType::Ipva
    }

    fn calculate_interest(&self, original_amount: Money, days_overdue: i64) -> Result<Money, DomainError> {
        if days_overdue <= 0 {
            return Ok(Money::zero());
        }
        let cents = original_amount.amount_in_cents();
        // 0.33% = 33 / 10000 por dia
        let calculated_numerator = cents * 33 * days_overdue;
        // Teto de 20% = 2000 / 10000
        let cap_numerator = cents * 2_000;
        let applied_numerator = calculated_numerator.min(cap_numerator);

        let rounded_cents = HalfUpRounder::round(applied_numerator, 10_000)?;
        Ok(Money::from_cents(rounded_cents))
    }
}

/// Política de Juros para Licenciamento: 0,33% ao dia limitado ao teto de 20% antes do arredondamento HALF_UP.
pub struct LicenciamentoInterestPolicy;

impl DebtInterestPolicy for LicenciamentoInterestPolicy {
    fn supports(&self) -> DebtType {
        DebtType::Licenciamento
    }

    fn calculate_interest(&self, original_amount: Money, days_overdue: i64) -> Result<Money, DomainError> {
        if days_overdue <= 0 {
            return Ok(Money::zero());
        }
        let cents = original_amount.amount_in_cents();
        let calculated_numerator = cents * 33 * days_overdue;
        let cap_numerator = cents * 2_000;
        let applied_numerator = calculated_numerator.min(cap_numerator);

        let rounded_cents = HalfUpRounder::round(applied_numerator, 10_000)?;
        Ok(Money::from_cents(rounded_cents))
    }
}

/// Política de Juros para Multa: 1,00% ao dia sem teto, com arredondamento HALF_UP.
pub struct MultaInterestPolicy;

impl DebtInterestPolicy for MultaInterestPolicy {
    fn supports(&self) -> DebtType {
        DebtType::Multa
    }

    fn calculate_interest(&self, original_amount: Money, days_overdue: i64) -> Result<Money, DomainError> {
        if days_overdue <= 0 {
            return Ok(Money::zero());
        }
        let cents = original_amount.amount_in_cents();
        // 1.00% = 1 / 100 por dia
        let calculated_numerator = cents * days_overdue;
        let rounded_cents = HalfUpRounder::round(calculated_numerator, 100)?;
        Ok(Money::from_cents(rounded_cents))
    }
}

#[derive(Clone)]
pub struct DebtInterestPolicyRegistry {
    policies: HashMap<DebtType, Arc<dyn DebtInterestPolicy>>,
}

impl DebtInterestPolicyRegistry {
    pub fn new(policies: Vec<Arc<dyn DebtInterestPolicy>>) -> Self {
        let mut map = HashMap::new();
        for p in policies {
            map.insert(p.supports(), p);
        }
        Self { policies: map }
    }

    pub fn with_defaults() -> Self {
        Self::new(vec![
            Arc::new(IpvaInterestPolicy),
            Arc::new(MultaInterestPolicy),
            Arc::new(LicenciamentoInterestPolicy),
        ])
    }

    pub fn get(&self, debt_type: DebtType) -> Option<&Arc<dyn DebtInterestPolicy>> {
        self.policies.get(&debt_type)
    }
}

impl Default for DebtInterestPolicyRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipva_interest_policy_with_20_percent_cap() {
        let policy = IpvaInterestPolicy;
        // 1500.00 com 121 dias (121 * 0.33% = 39.93% -> limitado a 20% = 300.00)
        let original = Money::from_decimal("1500.00").unwrap();
        let interest = policy.calculate_interest(original, 121).unwrap();
        assert_eq!(interest.to_decimal(), "300.00");

        // 10 dias de atraso em 1000.00 -> 10 * 0.33% = 3.30% -> 33.00
        let original_short = Money::from_decimal("1000.00").unwrap();
        let interest_short = policy.calculate_interest(original_short, 10).unwrap();
        assert_eq!(interest_short.to_decimal(), "33.00");
    }

    #[test]
    fn test_licenciamento_interest_policy() {
        let policy = LicenciamentoInterestPolicy;
        // 142.75 com 40 dias (40 * 0.33% = 13.2% -> 14275 * 1320 / 10000 = 1884.3 -> 18.84)
        let original = Money::from_decimal("142.75").unwrap();
        let interest = policy.calculate_interest(original, 40).unwrap();
        assert_eq!(interest.to_decimal(), "18.84");
    }

    #[test]
    fn test_multa_interest_policy_no_cap() {
        let policy = MultaInterestPolicy;
        // 300.50 com 85 dias (85 * 1% = 85% -> 30050 * 85 / 100 = 25542.5 -> HALF_UP = 25543 -> 255.43)
        let original = Money::from_decimal("300.50").unwrap();
        let interest = policy.calculate_interest(original, 85).unwrap();
        assert_eq!(interest.to_decimal(), "255.43");
    }

    #[test]
    fn test_zero_or_negative_days_overdue_returns_zero_interest() {
        let ipva = IpvaInterestPolicy;
        let multa = MultaInterestPolicy;
        let lic = LicenciamentoInterestPolicy;
        let amount = Money::from_decimal("500.00").unwrap();

        assert_eq!(ipva.calculate_interest(amount, 0).unwrap(), Money::zero());
        assert_eq!(ipva.calculate_interest(amount, -5).unwrap(), Money::zero());
        assert_eq!(multa.calculate_interest(amount, 0).unwrap(), Money::zero());
        assert_eq!(lic.calculate_interest(amount, 0).unwrap(), Money::zero());
    }
}
