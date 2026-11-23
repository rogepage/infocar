use crate::domain::debt::money::Money;
use crate::domain::payment::dto::{CreditCardInstallment, CreditCardOption};

#[derive(Clone, Default)]
pub struct CreditCardCalculator;

impl CreditCardCalculator {
    const ALLOWED_INSTALLMENTS: [u8; 3] = [1, 6, 12];
    const MONTHLY_INTEREST_RATE: f64 = 0.025; // 2,5% a.m.

    pub fn new() -> Self {
        Self
    }

    /// Calcula as opções de parcelamento no cartão de crédito:
    /// - `1x`: sem juros (`valor_base`)
    /// - `6x` e `12x`: Tabela Price com taxa composta de 2,5% a.m. e arredondamento `HALF_UP`.
    pub fn calculate(&self, base_amount: Money) -> CreditCardOption {
        let cents = base_amount.amount_in_cents();
        let mut installments = Vec::with_capacity(Self::ALLOWED_INSTALLMENTS.len());

        for &quantity in &Self::ALLOWED_INSTALLMENTS {
            let installment_amount = if quantity == 1 {
                base_amount
            } else {
                let i = Self::MONTHLY_INTEREST_RATE;
                let compounded = (1.0_f64 + i).powi(quantity as i32);
                let factor = (i * compounded) / (compounded - 1.0);
                let raw = (cents as f64) * factor;
                let installment_cents = (raw + 0.5).floor() as i64;
                Money::from_cents(installment_cents)
            };

            installments.push(CreditCardInstallment {
                quantity,
                installment_amount,
            });
        }

        CreditCardOption { installments }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_card_calculator_price_table_abc1234() {
        let calc = CreditCardCalculator::new();

        // TOTAL = 2355.93 -> 1x: 2355.93, 6x: 427.72, 12x: 229.67
        let total = calc.calculate(Money::from_decimal("2355.93").unwrap());
        assert_eq!(total.installments.len(), 3);
        assert_eq!(total.installments[0].quantity, 1);
        assert_eq!(total.installments[0].installment_amount.to_decimal(), "2355.93");
        assert_eq!(total.installments[1].quantity, 6);
        assert_eq!(total.installments[1].installment_amount.to_decimal(), "427.72");
        assert_eq!(total.installments[2].quantity, 12);
        assert_eq!(total.installments[2].installment_amount.to_decimal(), "229.67");

        // SOMENTE_IPVA = 1800.00 -> 1x: 1800.00, 6x: 326.79, 12x: 175.48
        let ipva = calc.calculate(Money::from_decimal("1800.00").unwrap());
        assert_eq!(ipva.installments[0].installment_amount.to_decimal(), "1800.00");
        assert_eq!(ipva.installments[1].installment_amount.to_decimal(), "326.79");
        assert_eq!(ipva.installments[2].installment_amount.to_decimal(), "175.48");

        // SOMENTE_MULTA = 555.93 -> 1x: 555.93, 6x: 100.93, 12x: 54.20
        let multa = calc.calculate(Money::from_decimal("555.93").unwrap());
        assert_eq!(multa.installments[0].installment_amount.to_decimal(), "555.93");
        assert_eq!(multa.installments[1].installment_amount.to_decimal(), "100.93");
        assert_eq!(multa.installments[2].installment_amount.to_decimal(), "54.20");
    }
}
