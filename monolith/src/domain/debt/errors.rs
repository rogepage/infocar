#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// Falha transitória (timeout, erro de rede, HTTP 5xx) -> Aciona retry com backoff linear
    #[error("{0}")]
    Unavailable(String),

    /// Falha de contrato/formato (JSON/XML/CSV corrompido) -> Fail-fast (sem retry, aciona fallback imediato)
    #[error("{0}")]
    InvalidResponse(String),

    /// Status HTTP inesperado não-5xx (ex: 404/403 do provider) -> Tratado como falha para fallback
    #[error("{0}")]
    UnexpectedStatus(String),
}

impl ProviderError {
    pub fn is_retriable(&self) -> bool {
        matches!(self, ProviderError::Unavailable(_))
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Invalid monetary amount string: {0}")]
    InvalidMoneyFormat(String),

    #[error("Denominator must be positive, got: {0}")]
    InvalidDenominator(i64),

    #[error("Unknown debt type: {0}")]
    UnknownDebtType(String),

    #[error("All vehicle debt providers failed: {0}")]
    AllProvidersUnavailable(String),
}
