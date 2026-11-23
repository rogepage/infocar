use crate::domain::debt::errors::DomainError;
use crate::domain::debt::money::Money;
use crate::domain::debt::rounding::HalfUpRounder;
use crate::domain::payment::dto::PixOption;

#[derive(Clone, Default)]
pub struct PixCalculator;

impl PixCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calcula o pagamento via PIX aplicando 5% de desconto (`95 / 100`) com arredondamento `HALF_UP`.
    pub fn calculate(&self, base_amount: Money) -> Result<PixOption, DomainError> {
        let base_cents = base_amount.amount_in_cents();
        let discounted_cents = HalfUpRounder::round(base_cents * 95, 100)?;
        Ok(PixOption {
            total_with_discount: Money::from_cents(discounted_cents),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pix_calculator_5_percent_discount_half_up() {
        let calc = PixCalculator::new();

        // 2355.93 * 0.95 = 2238.1335 -> 2238.13
        let base = Money::from_decimal("2355.93").unwrap();
        assert_eq!(
            calc.calculate(base).unwrap().total_with_discount.to_decimal(),
            "2238.13"
        );

        // 1800.00 * 0.95 = 1710.00
        let ipva = Money::from_decimal("1800.00").unwrap();
        assert_eq!(
            calc.calculate(ipva).unwrap().total_with_discount.to_decimal(),
            "1710.00"
        );

        // 555.93 * 0.95 = 528.1335 -> 528.13
        let multa = Money::from_decimal("555.93").unwrap();
        assert_eq!(
            calc.calculate(multa).unwrap().total_with_discount.to_decimal(),
            "528.13"
        );
    }
}
