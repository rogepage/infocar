# Implementation Plan: Monólito Infocar em Rust com Provedores Satélites em PHP

**Branch**: `001-rust-monolith-core` | **Data**: 2026-11-23 | **Spec**: [spec.md](spec.md)  
**Input**: Especificação da feature em `specs/001-rust-monolith-core/spec.md`

---

## Summary

Este plano detalha a construção do monólito principal do **Infocar** em **Rust** (`axum`, `tokio`, `reqwest`, `serde`, `chrono`, `roxmltree`, `askama`, `tracing`), substituindo o monólito Laravel/PHP enquanto mantém os 4 containers satélites (`provider-rest`, `provider-soap`, `provider-csv` e `payment-provider`) em **PHP**. A arquitetura preserva integralmente a separação hexagonal (Domínio Puro, Aplicação, Infraestrutura e Entrega HTTP/CLI), precisão financeira em centavos inteiros (`i64`) com arredondamento `HALF_UP`, resiliência *First-Success-Wins* e contratos HTTP/JSON/Logs 100% compatíveis com o projeto `cardok`.

---

## Technical Context

**Language/Version**: Rust 1.85+ (Edition 2024) no `monolith/`; PHP 8.4 nos containers satélites (`provider-*`)  
**Primary Dependencies**: `axum 0.8`, `tokio 1.43`, `tower / tower-http`, `reqwest 0.12`, `serde / serde_json`, `chrono 0.4`, `roxmltree 0.20`, `csv 1.3`, `regex 1.11`, `uuid 1.12`, `thiserror 2.0`, `async-trait 0.1`, `tracing / tracing-subscriber`, `askama 0.12`  
**Storage**: N/A (operação *stateless*; métricas em contadores atômicos `AtomicU64` e cache de healthcheck em memória com `RwLock`)  
**Testing**: `cargo test` (testes unitários de domínio, aplicação, parsers e testes de integração HTTP com `wiremock` e `tower::ServiceExt`)  
**Target Platform**: Docker Compose / Linux Containers (`monolith` compilado via multi-stage build em Rust + 4 containers PHP satélites)  
**Project Type**: Web Service + Web UI + CLI com Arquitetura Hexagonal (Ports & Adapters)  
**Performance Goals**: Latência interna de domínio/orquestração `< 1ms`; cache hit em `/api/health/integrations` `< 2ms`; timeout por tentativa externa configurado em `2s`  
**Constraints**:
- Princípio I: `monolith/src/domain` puro sem dependências de HTTP, JSON externo, XML ou frameworks web.
- Princípio II: Valores monetários manipulados estritamente via `Money(i64)` em centavos inteiros, sem `f64` para valores monetários e com arredondamento `HALF_UP`.
- Princípio III: Resiliência *First-Success-Wins* com timeout, retry linear (`attempt * 100ms`), fail-fast em erro de contrato e fallback transparente.
- Princípio VIII: Documentação e artefatos de engenharia redigidos em pt-BR.

---

## Constitution Check

*GATE: Avaliação pré-design e pós-design aprovada segundo a Constituição v2.0.0.*

| Princípio | Status | Justificativa |
|---|---|---|
| **I. Arquitetura e Isolamento do Domínio** | ✅ PASS | `src/domain` contém apenas tipos puros de negócio e traits (`Clock`, `VehicleDebtProvider`, `DebtInterestPolicy`). Os adaptadores residem em `src/infrastructure/providers/`. |
| **II. Precisão Financeira** | ✅ PASS | `Money(i64)` e `HalfUpRounder` operam exclusivamente com aritmética inteira de 64 bits. `f64` é restrito ao fator adimensional da Tabela Price em `CreditCardCalculator`. |
| **III. Integrações e Resiliência** | ✅ PASS | `ProviderResolver`, `ProviderExecutor` e `VehicleDebtService` implementam retry linear, fail-fast para `InvalidResponse` e fallback *First-Success-Wins*. Zero débitos não dispara fallback. |
| **IV. Simulação de Pagamentos** | ✅ PASS | PIX (5% de desconto) e Cartão (`1x`, `6x`, `12x` a 2,5% a.m. via Price) agrupados em `TOTAL` e `SOMENTE_<TIPO>` na ordem de aparição. |
| **V. Determinismo Temporal** | ✅ PASS | `FixedClock` injetado com `2024-05-10T00:00:00Z` e cálculos de dias vencidos realizados em UTC. |
| **VI. Observabilidade e Privacidade** | ✅ PASS | Propagação de `X-Request-ID`, mascaramento via `PlateMasker`, métricas atômicas em `/api/metrics` e logs JSON estruturados em `stdout` e `storage/logs/laravel.log`. |
| **VII. Testes** | ✅ PASS | Suíte completa de testes unitários e de integração planejada em `cargo test`. |
| **VIII. Artefatos de Engenharia** | ✅ PASS | Especificações, planos, modelo de dados, contratos e tarefas redigidos em pt-BR. |

---

## Project Structure

### Documentation (this feature)

```text
specs/001-rust-monolith-core/
├── spec.md                  # Especificação de requisitos e histórias de usuário
├── plan.md                  # Este plano de implementação técnica
├── data-model.md            # Modelo de dados canônico e tipagem Rust
├── contracts/
│   └── api-contracts.md     # Contratos HTTP da API, providers PHP e logs JSON
└── tasks.md                 # Tarefas ordenadas (TDD) para implementação
```

### Source Code Layout

```text
infocar/
├── AGENTS.md
├── CLAUDE.md
├── README.md
├── Makefile
├── docker-compose.yml
├── .env.example
├── scripts/
│   ├── demo-fallback.sh
│   └── format-demo-logs.py
├── monolith/                                        # Monólito em Rust
│   ├── Cargo.toml
│   ├── Dockerfile
│   ├── templates/
│   │   └── lookup.html
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config.rs
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── debt/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── money.rs
│   │   │   │   ├── rounding.rs
│   │   │   │   ├── debt_type.rs
│   │   │   │   ├── entities.rs
│   │   │   │   ├── clock.rs
│   │   │   │   ├── contracts.rs
│   │   │   │   ├── errors.rs
│   │   │   │   ├── policies.rs
│   │   │   │   └── calculation_service.rs
│   │   │   └── payment/
│   │   │       ├── mod.rs
│   │   │       ├── dto.rs
│   │   │       ├── pix_calculator.rs
│   │   │       ├── credit_card_calculator.rs
│   │   │       └── simulator.rs
│   │   ├── application/
│   │   │   ├── mod.rs
│   │   │   ├── plate_masker.rs
│   │   │   └── vehicle_debt/
│   │   │       ├── mod.rs
│   │   │       ├── resolver.rs
│   │   │       ├── executor.rs
│   │   │       ├── service.rs
│   │   │       ├── result.rs
│   │   │       └── use_case.rs
│   │   ├── infrastructure/
│   │   │   ├── mod.rs
│   │   │   ├── observability/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── metrics.rs
│   │   │   │   └── logger.rs
│   │   │   └── providers/
│   │   │       ├── mod.rs
│   │   │       ├── rest.rs
│   │   │       ├── soap.rs
│   │   │       └── ai_csv.rs
│   │   ├── http/
│   │   │   ├── mod.rs
│   │   │   ├── state.rs
│   │   │   ├── routes.rs
│   │   │   ├── middleware/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── request_id.rs
│   │   │   │   ├── security_headers.rs
│   │   │   │   └── rate_limit.rs
│   │   │   └── handlers/
│   │   │       ├── mod.rs
│   │   │       ├── vehicle_debts.rs
│   │   │       ├── health.rs
│   │   │       ├── metrics.rs
│   │   │       └── web_ui.rs
│   │   └── cli/
│   │       ├── mod.rs
│   │       └── check_services.rs
│   └── tests/
│       ├── api_integration_test.rs
│       ├── fallback_resilience_test.rs
│       └── observability_cache_test.rs
├── provider-rest/                                   # Container satélite em PHP (porta 8001)
├── provider-soap/                                   # Container satélite em PHP (porta 8002)
├── provider-csv/                                    # Container satélite em PHP (porta 8003)
└── payment-provider/                                # Container satélite em PHP (rede interna)
```
