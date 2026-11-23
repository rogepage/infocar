use crate::domain::debt::errors::DomainError;
use std::fmt;

/// Value Object imutável que representa valores monetários estritamente em centavos inteiros (`i64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Money {
    amount_in_cents: i64,
}

impl Money {
    pub const fn from_cents(cents: i64) -> Self {
        Self {
            amount_in_cents: cents,
        }
    }

    pub const fn zero() -> Self {
        Self { amount_in_cents: 0 }
    }

    /// Converte uma string decimal (ex.: `"1500.00"`, `"300.5"`, `"100"`, `"-10.05"`)
    /// para `Money` usando exclusivamente parsing de inteiros (`i64`), sem `f64`.
    pub fn from_decimal(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidMoneyFormat(raw.to_string()));
        }

        let (is_negative, unsigned) = if let Some(rest) = trimmed.strip_prefix('-') {
            (true, rest)
        } else {
            (false, trimmed)
        };

        if unsigned.is_empty() {
            return Err(DomainError::InvalidMoneyFormat(raw.to_string()));
        }

        let mut parts = unsigned.split('.');
        let units_str = parts.next().unwrap_or("");
        let cents_opt = parts.next();

        // Se houver mais de um ponto decimal, formato inválido
        if parts.next().is_some() {
            return Err(DomainError::InvalidMoneyFormat(raw.to_string()));
        }

        if units_str.is_empty() || !units_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::InvalidMoneyFormat(raw.to_string()));
        }

        let units: i64 = units_str
            .parse::<i64>()
            .map_err(|_| DomainError::InvalidMoneyFormat(raw.to_string()))?;

        let cents: i64 = match cents_opt {
            None => 0,
            Some(c_str) => {
                if c_str.is_empty()
                    || c_str.len() > 2
                    || !c_str.chars().all(|c| c.is_ascii_digit())
                {
                    return Err(DomainError::InvalidMoneyFormat(raw.to_string()));
                }
                let parsed = c_str
                    .parse::<i64>()
                    .map_err(|_| DomainError::InvalidMoneyFormat(raw.to_string()))?;
                if c_str.len() == 1 {
                    parsed * 10
                } else {
                    parsed
                }
            }
        };

        let total_cents = units
            .checked_mul(100)
            .and_then(|u| u.checked_add(cents))
            .ok_or_else(|| DomainError::InvalidMoneyFormat(raw.to_string()))?;

        let signed_cents = if is_negative {
            -total_cents
        } else {
            total_cents
        };

        Ok(Self::from_cents(signed_cents))
    }

    pub const fn amount_in_cents(&self) -> i64 {
        self.amount_in_cents
    }

    pub const fn cents(&self) -> i64 {
        self.amount_in_cents
    }

    pub const fn add(self, other: Self) -> Self {
        Self {
            amount_in_cents: self.amount_in_cents + other.amount_in_cents,
        }
    }

    pub const fn subtract(self, other: Self) -> Self {
        Self {
            amount_in_cents: self.amount_in_cents - other.amount_in_cents,
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.amount_in_cents == 0
    }

    pub const fn is_positive(&self) -> bool {
        self.amount_in_cents > 0
    }

    pub const fn is_negative(&self) -> bool {
        self.amount_in_cents < 0
    }

    pub fn to_decimal(&self) -> String {
        let sign = if self.amount_in_cents < 0 { "-" } else { "" };
        let abs = self.amount_in_cents.unsigned_abs();
        let units = abs / 100;
        let cents = abs % 100;
        format!("{}{}.{:02}", sign, units, cents)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_decimal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_from_cents_and_to_decimal() {
        assert_eq!(Money::from_cents(150_000).to_decimal(), "1500.00");
        assert_eq!(Money::from_cents(30_050).to_decimal(), "300.50");
        assert_eq!(Money::from_cents(5).to_decimal(), "0.05");
        assert_eq!(Money::from_cents(0).to_decimal(), "0.00");
        assert_eq!(Money::from_cents(-1005).to_decimal(), "-10.05");
    }

    #[test]
    fn test_money_from_decimal_valid_inputs() {
        assert_eq!(
            Money::from_decimal("1500.00").unwrap(),
            Money::from_cents(150_000)
        );
        assert_eq!(
            Money::from_decimal(" 300.5 ").unwrap(),
            Money::from_cents(30_050)
        );
        assert_eq!(
            Money::from_decimal("142.75").unwrap(),
            Money::from_cents(14_275)
        );
        assert_eq!(
            Money::from_decimal("100").unwrap(),
            Money::from_cents(10_000)
        );
        assert_eq!(
            Money::from_decimal("-10.05").unwrap(),
            Money::from_cents(-1005)
        );
        assert_eq!(Money::from_decimal("-0.00").unwrap(), Money::from_cents(0));
    }

    #[test]
    fn test_money_from_decimal_rejects_invalid_formats() {
        assert!(Money::from_decimal("").is_err());
        assert!(Money::from_decimal("abc").is_err());
        assert!(Money::from_decimal("12.345").is_err());
        assert!(Money::from_decimal("12.").is_err());
        assert!(Money::from_decimal(".50").is_err());
        assert!(Money::from_decimal("12,50").is_err());
        assert!(Money::from_decimal("12.50.00").is_err());
    }

    #[test]
    fn test_money_arithmetic_and_predicates() {
        let a = Money::from_cents(150_000);
        let b = Money::from_cents(30_050);
        assert_eq!(a.add(b), Money::from_cents(180_050));
        assert_eq!(a.subtract(b), Money::from_cents(119_950));
        assert!(a.is_positive());
        assert!(!a.is_zero());
        assert!(Money::zero().is_zero());
        assert!(Money::from_cents(-1).is_negative());
    }
}
