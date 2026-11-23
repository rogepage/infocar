use crate::domain::debt::errors::DomainError;

pub struct HalfUpRounder;

impl HalfUpRounder {
    /// Arredonda a fração racional `numerator / denominator` utilizando `HALF_UP`
    /// estritamente com aritmética inteira de 64 bits (zero uso de ponto flutuante).
    pub fn round(numerator: i64, denominator: i64) -> Result<i64, DomainError> {
        if denominator <= 0 {
            return Err(DomainError::InvalidDenominator(denominator));
        }

        let half = denominator / 2;
        if numerator >= 0 {
            Ok((numerator + half) / denominator)
        } else {
            Ok(-((-numerator + half) / denominator))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_up_rounding_positive_boundaries() {
        // .49 -> round down
        assert_eq!(HalfUpRounder::round(149, 100).unwrap(), 1);
        // .50 -> round up (away from zero)
        assert_eq!(HalfUpRounder::round(150, 100).unwrap(), 2);
        // .51 -> round up
        assert_eq!(HalfUpRounder::round(151, 100).unwrap(), 2);
        // exact division
        assert_eq!(HalfUpRounder::round(200, 100).unwrap(), 2);
        assert_eq!(HalfUpRounder::round(0, 100).unwrap(), 0);
    }

    #[test]
    fn test_half_up_rounding_negative_boundaries() {
        // -1.49 -> -1
        assert_eq!(HalfUpRounder::round(-149, 100).unwrap(), -1);
        // -1.50 -> -2 (half away from zero)
        assert_eq!(HalfUpRounder::round(-150, 100).unwrap(), -2);
        // -1.51 -> -2
        assert_eq!(HalfUpRounder::round(-151, 100).unwrap(), -2);
    }

    #[test]
    fn test_half_up_rejects_non_positive_denominator() {
        assert_eq!(
            HalfUpRounder::round(100, 0),
            Err(DomainError::InvalidDenominator(0))
        );
        assert_eq!(
            HalfUpRounder::round(100, -10),
            Err(DomainError::InvalidDenominator(-10))
        );
    }
}
