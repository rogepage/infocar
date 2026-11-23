# Fase 1 — Domínio Financeiro Puro em Rust (`monolith/src/domain/`)

## Objetivo
Implementar o núcleo de domínio do **Infocar** em Rust puro (`src/domain/`), sem qualquer dependência de HTTP (`axum`, `reqwest`), XML, CSV ou infraestrutura, garantindo 100% de precisão centesimal (`i64`) e arredondamento `HALF_UP`.

---

## 1. Módulos a Criar

```text
monolith/src/domain/
├── mod.rs
├── debt/
│   ├── mod.rs
│   ├── money.rs                 # Value Object Money(i64)
│   ├── rounding.rs              # HalfUpRounder
│   ├── debt_type.rs             # Enum DebtType (Ipva, Multa, Licenciamento)
│   ├── entities.rs              # Debt, CalculatedDebt, CalculatedVehicleDebts, ProviderDebtResponse
│   ├── clock.rs                 # Trait Clock + FixedClock (2024-05-10T00:00:00Z)
│   ├── contracts.rs             # Trait VehicleDebtProvider
│   ├── errors.rs                # DomainError e ProviderError
│   ├── policies.rs              # Trait DebtInterestPolicy + Ipva, Multa, Licenciamento + Registry
│   └── calculation_service.rs   # DebtCalculationService
└── payment/
    ├── mod.rs
    ├── dto.rs                   # PaymentSimulationResult, PaymentOption, PixOption, CreditCardOption, CreditCardInstallment
    ├── pix_calculator.rs        # PixCalculator (5% de desconto)
    ├── credit_card_calculator.rs# CreditCardCalculator (1x, 6x, 12x Tabela Price 2.5% a.m.)
    └── simulator.rs             # PaymentSimulator (TOTAL e SOMENTE_<TIPO>)
```

---

## 2. Regras de Implementação

### 2.1. `Money` (`src/domain/debt/money.rs`)
- Armazena internamente `amount_in_cents: i64`.
- `Money::from_cents(cents: i64) -> Self`
- `Money::from_decimal(raw: &str) -> Result<Self, DomainError>`:
  - Trim da string; valida formato `^-?\d+(\.\d{1,2})?$`.
  - Faz split no `.`, converte unidades e centavos (com `pad` à direita de `'0'` até 2 dígitos) usando apenas inteiros (`i64`). **Proibido usar `parse::<f64>()`**.
- `Money::to_decimal(&self) -> String`:
  - Formata como `"{sign}{units}.{cents:02}"` (ex.: `150000` → `"1500.00"`).
- Operações: `add(Money)`, `subtract(Money)`, `is_zero()`, `is_positive()`, `is_negative()`, `amount_in_cents()`.

### 2.2. `HalfUpRounder` (`src/domain/debt/rounding.rs`)
- `HalfUpRounder::round(numerator: i64, denominator: i64) -> Result<i64, DomainError>`:
  - Rejeita `denominator <= 0`.
  - Para `numerator >= 0`: `(numerator + (denominator / 2)) / denominator`.
  - Para `numerator < 0`: `-((-numerator + (denominator / 2)) / denominator)`.

### 2.3. `DebtType` (`src/domain/debt/debt_type.rs`)
- Variantes: `Ipva`, `Multa`, `Licenciamento`.
- `DebtType::try_from_normalized(raw: &str) -> Option<Self>`: faz `trim().to_uppercase()` e mapeia `"IPVA"`, `"MULTA"`, `"LICENCIAMENTO"`.
- `as_str(&self) -> &'static str`: retorna `"IPVA"`, `"MULTA"` ou `"LICENCIAMENTO"`.

### 2.4. Políticas de Juros (`src/domain/debt/policies.rs`)
- **`IpvaInterestPolicy` e `LicenciamentoInterestPolicy`**:
  - Se `days_overdue <= 0`, retorna `Money::from_cents(0)`.
  - `calculated_numerator = cents * 33 * (days_overdue as i64)`
  - `cap_numerator = cents * 2000`
  - `applied_numerator = calculated_numerator.min(cap_numerator)`
  - `rounded_cents = HalfUpRounder::round(applied_numerator, 10_000)`
- **`MultaInterestPolicy`**:
  - Se `days_overdue <= 0`, retorna `Money::from_cents(0)`.
  - `calculated_numerator = cents * 1 * (days_overdue as i64)`
  - `rounded_cents = HalfUpRounder::round(calculated_numerator, 100)`

### 2.5. `DebtCalculationService` (`src/domain/debt/calculation_service.rs`)
- Recebe `Arc<dyn Clock>` (fixado em `2024-05-10T00:00:00Z`) e `DebtInterestPolicyRegistry`.
- Se encontrar `Debt` cujo `type` não seja reconhecido por `DebtType::try_from_normalized`, retorna `Err(DomainError::UnknownDebtType(raw_type))`.
- Calcula `days_overdue = (ref_date - due_date).num_days().max(0)` em UTC.

### 2.6. Simulação de Pagamentos (`src/domain/payment/`)
- **`PixCalculator`**: `HalfUpRounder::round(base_cents * 95, 100)`.
- **`CreditCardCalculator`**:
  - Parcelas permitidas: `[1, 6, 12]`.
  - `1x`: retorna `base_amount`.
  - `6x` e `12x`:
    ```rust
    let i: f64 = 0.025;
    let compounded = (1.0_f64 + i).powi(quantity as i32);
    let factor = (i * compounded) / (compounded - 1.0);
    let raw = (cents as f64) * factor;
    let installment_cents = (raw + 0.5).floor() as i64;
    ```
- **`PaymentSimulator`**:
  - Se `debts.is_empty()` ou `total_updated.is_zero()`, retorna `opcoes: vec![]`.
  - Gera `TOTAL` primeiro, seguido de `SOMENTE_{TIPO}` agrupando na ordem de primeira aparição dos tipos na lista de débitos.

---

## 3. Critérios de Aceite da Fase 1
- Placa `ABC1234` com `IPVA` (`1500.00`, venc. `2024-01-10`, 121 dias) e `MULTA` (`300.50`, venc. `2024-02-15`, 85 dias) na data `2024-05-10` deve produzir exatamente:
  - `IPVA` atualizado: `"1800.00"` (teto de 20% atingido)
  - `MULTA` atualizada: `"555.93"`
  - `total_original`: `"1800.50"`, `total_atualizado`: `"2355.93"`
  - `TOTAL` → PIX: `"2238.13"`, Cartão 1x: `"2355.93"`, 6x: `"427.72"`, 12x: `"229.67"`
  - `SOMENTE_IPVA` → PIX: `"1710.00"`, Cartão 1x: `"1800.00"`, 6x: `"326.79"`, 12x: `"175.48"`
  - `SOMENTE_MULTA` → PIX: `"528.13"`, Cartão 1x: `"555.93"`, 6x: `"100.93"`, 12x: `"54.20"`
