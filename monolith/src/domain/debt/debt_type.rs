use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebtType {
    Ipva,
    Multa,
    Licenciamento,
}

impl DebtType {
    pub fn try_from_normalized(raw: &str) -> Option<Self> {
        match raw.trim().to_uppercase().as_str() {
            "IPVA" => Some(Self::Ipva),
            "MULTA" => Some(Self::Multa),
            "LICENCIAMENTO" => Some(Self::Licenciamento),
            _ => None,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ipva => "IPVA",
            Self::Multa => "MULTA",
            Self::Licenciamento => "LICENCIAMENTO",
        }
    }
}

impl fmt::Display for DebtType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_type_normalization() {
        assert_eq!(DebtType::try_from_normalized("ipva"), Some(DebtType::Ipva));
        assert_eq!(
            DebtType::try_from_normalized(" MULTA "),
            Some(DebtType::Multa)
        );
        assert_eq!(
            DebtType::try_from_normalized("Licenciamento"),
            Some(DebtType::Licenciamento)
        );
        assert_eq!(DebtType::try_from_normalized("DPVAT"), None);
    }
}
