# Implementation Plan: Fase 1 — Domínio Financeiro Puro (`001-dominio-financeiro`)

**Branch**: `001-dominio-financeiro` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Escopo Técnico da Fase 1

Construir a camada `monolith/src/domain/` contendo:
- `domain::debt::errors` (`DomainError`, `ProviderError`)
- `domain::debt::rounding` (`HalfUpRounder`)
- `domain::debt::money` (`Money(i64)`)
- `domain::debt::debt_type` (`DebtType`)
- `domain::debt::entities` (`Debt`, `ProviderDebtResponse`, `CalculatedDebt`, `CalculatedVehicleDebts`)
- `domain::debt::clock` (`Clock`, `FixedClock`)
- `domain::debt::contracts` (`VehicleDebtProvider` trait)
- `domain::debt::policies` (`DebtInterestPolicy`, `IpvaInterestPolicy`, `MultaInterestPolicy`, `LicenciamentoInterestPolicy`, `DebtInterestPolicyRegistry`)
- `domain::debt::calculation_service` (`DebtCalculationService`)
- `domain::payment::dto` (`PixOption`, `CreditCardInstallment`, `CreditCardOption`, `PaymentOption`, `PaymentSimulationResult`)
- `domain::payment::pix_calculator` (`PixCalculator`)
- `domain::payment::credit_card_calculator` (`CreditCardCalculator`)
- `domain::payment::simulator` (`PaymentSimulator`)

---

## 2. Gates da Constituição Verificados nesta Fase

- **Princípio I (Isolamento do Domínio)**: `monolith/src/domain/` depende apenas da standard library, `chrono`, `thiserror` e `async-trait` (apenas para a assinatura da porta `VehicleDebtProvider`).
- **Princípio II (Precisão Financeira)**: `Money(i64)` e `HalfUpRounder` operam 100% em inteiros; `f64` é utilizado exclusivamente para o fator adimensional da Tabela Price em `CreditCardCalculator`.
- **Princípio IV (Simulação de Pagamentos)**: PIX 5% OFF e Cartão 1x, 6x, 12x (`2,5% a.m.`) agrupados em `TOTAL` e `SOMENTE_<TIPO>`.
- **Princípio V (Determinismo Temporal)**: `FixedClock` fixado em `2024-05-10T00:00:00Z`.
