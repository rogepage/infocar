# Tasks: Fase 1 — Domínio Financeiro Puro (`001-dominio-financeiro`)

- [x] T001 Inicializar `monolith/Cargo.toml`, `monolith/src/config.rs` e `monolith/src/lib.rs`
- [x] T002 [P] Implementar `DomainError` e `ProviderError` em `monolith/src/domain/debt/errors.rs`
- [x] T003 [P] Implementar `HalfUpRounder` e testes unitários em `monolith/src/domain/debt/rounding.rs`
- [x] T004 Implementar `Money(i64)` (`from_cents`, `from_decimal`, `to_decimal`, soma/subtração) e testes unitários em `monolith/src/domain/debt/money.rs`
- [x] T005 [P] Implementar `DebtType` em `monolith/src/domain/debt/debt_type.rs` e entidades (`Debt`, `ProviderDebtResponse`, `CalculatedDebt`, `CalculatedVehicleDebts`) em `monolith/src/domain/debt/entities.rs`
- [x] T006 [P] Implementar `Clock` e `FixedClock` (`2024-05-10T00:00:00Z`) em `monolith/src/domain/debt/clock.rs` e a porta `VehicleDebtProvider` em `monolith/src/domain/debt/contracts.rs`
- [x] T007 Implementar políticas de juros (`IpvaInterestPolicy`, `MultaInterestPolicy`, `LicenciamentoInterestPolicy`, `DebtInterestPolicyRegistry`) e testes unitários em `monolith/src/domain/debt/policies.rs`
- [x] T008 Implementar `DebtCalculationService` e testes unitários em `monolith/src/domain/debt/calculation_service.rs`
- [x] T009 [P] Implementar DTOs de pagamento em `monolith/src/domain/payment/dto.rs`
- [x] T010 [P] Implementar `PixCalculator` (5% OFF via `HalfUpRounder`) e testes unitários em `monolith/src/domain/payment/pix_calculator.rs`
- [x] T011 [P] Implementar `CreditCardCalculator` (1x, 6x, 12x Tabela Price 2,5% a.m.) e testes unitários em `monolith/src/domain/payment/credit_card_calculator.rs`
- [x] T012 Implementar `PaymentSimulator` (`TOTAL` e `SOMENTE_<TIPO>`) e testes unitários em `monolith/src/domain/payment/simulator.rs`
