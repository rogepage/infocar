# Data Model: Monólito Infocar em Rust

**Feature**: `001-rust-monolith-core`  
**Data**: 2026-11-23  
**Status**: Aprovado  

---

## 1. Tipos Fundamentais de Domínio (`src/domain/debt/`)

### 1.1. `Money`
Representa um valor monetário imutável em centavos inteiros de 64 bits (`i64`).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Money {
    amount_in_cents: i64,
}
```
- **Invariantes**:
  - `from_decimal(s: &str)` aceita apenas strings que satisfaçam `^-?\d+(\.\d{1,2})?$`.
  - Nunca converte para `f32` ou `f64`.
  - `to_decimal(&self) -> String` formata sempre com ponto e duas casas decimais (`"1500.00"`, `"-10.05"`).

### 1.2. `DebtType`
Categorias de débitos veiculares homologadas pelo domínio.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebtType {
    Ipva,
    Multa,
    Licenciamento,
}
```

### 1.3. `Debt` (Entidade Canônica de Entrada)
Representa um débito bruto já normalizado por qualquer adaptador (`REST`, `SOAP` ou `AI/CSV`).

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Debt {
    pub raw_type: String,
    pub amount: Money,
    pub due_date: NaiveDate,
}
```

### 1.4. `ProviderDebtResponse`
Envelope canônico retornado por qualquer implementação de `VehicleDebtProvider`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDebtResponse {
    pub plate: String,
    pub debts: Vec<Debt>,
    pub provider: String, // "rest" | "soap" | "ai"
}
```

### 1.5. `CalculatedDebt` e `CalculatedVehicleDebts`
Resultado do processamento financeiro de juros moratórios por `DebtCalculationService`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatedDebt {
    pub debt_type: DebtType,
    pub original_amount: Money,
    pub updated_amount: Money,
    pub interest_amount: Money,
    pub due_date: NaiveDate,
    pub days_overdue: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculatedVehicleDebts {
    pub plate: String,
    pub debts: Vec<CalculatedDebt>,
    pub total_original: Money,
    pub total_updated: Money,
}
```

---

## 2. Tipos do Domínio de Pagamentos (`src/domain/payment/`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixOption {
    pub total_with_discount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardInstallment {
    pub quantity: u8, // 1, 6 ou 12
    pub installment_amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardOption {
    pub installments: Vec<CreditCardInstallment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentOption {
    pub option_type: String, // "TOTAL" | "SOMENTE_IPVA" | "SOMENTE_MULTA" | "SOMENTE_LICENCIAMENTO"
    pub base_amount: Money,
    pub pix: PixOption,
    pub credit_card: CreditCardOption,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSimulationResult {
    pub options: Vec<PaymentOption>,
}
```

---

## 3. Hierarquia de Erros (`src/domain/debt/errors.rs`)

```rust
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
```
