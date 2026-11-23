# Tasks: Recriação do Monólito em Rust (Infocar) + Providers em PHP

**Input**: Documentos de design em `/specs/001-rust-monolith-core/` (`spec.md`, `plan.md`, `data-model.md`, `contracts/api-contracts.md`) e `/docs/rust-phases/`  
**Prerequisites**: `plan.md`, `spec.md`, `data-model.md`, `contracts/api-contracts.md`  
**Organization**: Tarefas agrupadas por fase e User Story para permitir desenvolvimento TDD e validação independente.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode ser executada em paralelo (arquivos distintos, sem dependências bloqueantes)
- **[Story]**: User Story associada (`[US1]`, `[US2]`, `[US3]`)

---

## Phase 1: Setup & Infraestrutura de Providers PHP

**Purpose**: Inicializar o projeto Cargo em `monolith/` e preparar os containers satélites em PHP (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`).

- [ ] T001 Copiar os diretórios dos providers satélites em PHP (`provider-rest/`, `provider-soap/`, `provider-csv/`, `payment-provider/`) e `scripts/` de `/Users/roger/Desktop/docker/cardok` para `/Users/roger/Desktop/docker/infocar`
- [ ] T002 Inicializar o pacote Rust em `monolith/Cargo.toml` com todas as dependências (`axum`, `tokio`, `reqwest`, `serde`, `serde_json`, `chrono`, `roxmltree`, `csv`, `regex`, `uuid`, `thiserror`, `async-trait`, `tracing`, `tracing-subscriber`, `askama`, `wiremock`) e estrutura base `monolith/src/lib.rs` e `monolith/src/main.rs`
- [ ] T003 [P] Implementar `monolith/src/config.rs` lendo as variáveis de ambiente (`PROVIDER_REST_URL`, `PROVIDER_SOAP_URL`, `PROVIDER_AI_URL`, `PROVIDER_CSV_URL`, `PAYMENT_PROVIDER_URL`, `PROVIDER_ORDER`, `PROVIDER_TIMEOUT`, `PROVIDER_RETRIES`, `PROVIDER_BACKOFF_MS`)

---

## Phase 2: User Story 1 - Domínio Financeiro Determinístico e Simulação de Pagamentos (Priority: P1) 🎯 MVP

**Goal**: Implementar o domínio puro em `monolith/src/domain/` com precisão em centavos inteiros (`i64`), arredondamento `HALF_UP`, políticas de juros (IPVA, Multa, Licenciamento) e simulador de pagamentos (PIX 5% e Cartão Tabela Price 1x, 6x, 12x).

**Independent Test**: `cd monolith && cargo test domain::` passando 100% com paridade exata ao centavo em relação ao `cardok`.

### Implementation & Unit Tests for User Story 1

- [ ] T004 [P] [US1] Implementar `DomainError` e `ProviderError` em `monolith/src/domain/debt/errors.rs`
- [ ] T005 [P] [US1] Implementar `HalfUpRounder` e seus testes unitários em `monolith/src/domain/debt/rounding.rs`
- [ ] T006 [US1] Implementar o Value Object `Money(i64)` (`from_cents`, `from_decimal`, `to_decimal`, soma/subtração) e testes unitários em `monolith/src/domain/debt/money.rs`
- [ ] T007 [P] [US1] Implementar o enum `DebtType` (`Ipva`, `Multa`, `Licenciamento`) em `monolith/src/domain/debt/debt_type.rs` e as entidades canônicas (`Debt`, `ProviderDebtResponse`, `CalculatedDebt`, `CalculatedVehicleDebts`) em `monolith/src/domain/debt/entities.rs`
- [ ] T008 [P] [US1] Implementar a trait `Clock` e `FixedClock` (`2024-05-10T00:00:00Z`) em `monolith/src/domain/debt/clock.rs` e a trait `VehicleDebtProvider` em `monolith/src/domain/debt/contracts.rs`
- [ ] T009 [US1] Implementar `DebtInterestPolicy`, `IpvaInterestPolicy`, `MultaInterestPolicy`, `LicenciamentoInterestPolicy`, `DebtInterestPolicyRegistry` e testes unitários em `monolith/src/domain/debt/policies.rs`
- [ ] T010 [US1] Implementar `DebtCalculationService` e testes unitários (incluindo rejeição de `UnknownDebtType` e débitos não vencidos) em `monolith/src/domain/debt/calculation_service.rs`
- [ ] T011 [P] [US1] Implementar DTOs de pagamento (`PixOption`, `CreditCardInstallment`, `CreditCardOption`, `PaymentOption`, `PaymentSimulationResult`) em `monolith/src/domain/payment/dto.rs`
- [ ] T012 [P] [US1] Implementar `PixCalculator` (5% de desconto com `HalfUpRounder`) e testes unitários em `monolith/src/domain/payment/pix_calculator.rs`
- [ ] T013 [P] [US1] Implementar `CreditCardCalculator` (1x, 6x, 12x via Tabela Price a 2,5% a.m.) e testes unitários em `monolith/src/domain/payment/credit_card_calculator.rs`
- [ ] T014 [US1] Implementar `PaymentSimulator` (agrupamento `TOTAL` e `SOMENTE_<TIPO>` na ordem de aparição e retorno vazio para 0 débitos) e testes unitários em `monolith/src/domain/payment/simulator.rs`

**Checkpoint**: Domínio financeiro e de pagamentos 100% funcional, isolado e validado com `cargo test`.

---

## Phase 3: User Story 2 - Adaptadores de Provedores (REST, SOAP, AI/CSV) e Resiliência (Priority: P1)

**Goal**: Conectar o monólito Rust aos provedores externos em PHP com parsing defensivo (JSON, XML, CSV), retry linear com backoff, fail-fast e fallback *First-Success-Wins*.

**Independent Test**: `cd monolith && cargo test application:: infrastructure::` validando todos os parsers e cenários de retry/fallback com mocks HTTP (`wiremock`).

### Implementation & Tests for User Story 2

- [ ] T015 [P] [US2] Implementar `PlateMasker` (`ABC1234` → `ABC****`) e testes unitários em `monolith/src/application/plate_masker.rs`
- [ ] T016 [P] [US2] Implementar `SimpleMetricsRegistry` com contadores `AtomicU64` em `monolith/src/infrastructure/observability/metrics.rs` e logger JSON estruturado (`stdout` + arquivo `storage/logs/laravel.log`) em `monolith/src/infrastructure/observability/logger.rs`
- [ ] T017 [P] [US2] Implementar `RestVehicleDebtProvider` e testes unitários (sucesso, 0 débitos, JSON inválido, HTTP 500, timeout, propagação de `X-Request-ID`) em `monolith/src/infrastructure/providers/rest.rs`
- [ ] T018 [P] [US2] Implementar `SoapVehicleDebtProvider` e testes unitários (sucesso XML, tag auto-fechada `<debts/>`, XML malformado, HTTP 500, timeout) em `monolith/src/infrastructure/providers/soap.rs`
- [ ] T019 [P] [US2] Implementar `AiVehicleDebtProvider` (parser CSV defensivo com suporte a `,` e `;`, remoção de blocos Markdown ```` ```csv ````, comentários, descarte de categorias não suportadas com log `WARN` e falha em CSV corrompido) e testes unitários em `monolith/src/infrastructure/providers/ai_csv.rs`
- [ ] T020 [US2] Implementar `ProviderResolver` (registro de `rest`, `soap`, `ai`, `csv` e resolução de `PROVIDER_ORDER`) e testes unitários em `monolith/src/application/vehicle_debt/resolver.rs`
- [ ] T021 [US2] Implementar `ProviderExecutor` (com `Sleeper` trait injetável, retry linear `attempt * 100ms` para `Unavailable`, fail-fast para `InvalidResponse`, métricas e logs) e testes unitários em `monolith/src/application/vehicle_debt/executor.rs`
- [ ] T022 [US2] Implementar `VehicleDebtService` (fallback *First-Success-Wins*, métricas de fallback e erro `AllProvidersUnavailable`), `VehicleDebtConsultationResult` e `GetVehicleDebtsUseCase` em `monolith/src/application/vehicle_debt/`

**Checkpoint**: Camadas de Aplicação e Infraestrutura completas e testadas.

---

## Phase 4: User Story 3 - Entrega HTTP (Axum), Cache, Middlewares, Web UI e CLI (Priority: P2)

**Goal**: Expor as rotas HTTP, validação defensiva de payload, cache de 5s com `X-Cache`, interface Web e comando CLI `check-services`.

**Independent Test**: Executar testes de integração HTTP (`cargo test --test '*'`) cobrindo todos os endpoints e códigos de status.

### Implementation & Tests for User Story 3

- [ ] T023 [P] [US3] Implementar `RequestIdMiddleware` (validação/geração de `X-Request-ID`, log `request.received` e métrica `vehicle_debt_requests_total`) em `monolith/src/http/middleware/request_id.rs`
- [ ] T024 [P] [US3] Implementar `SecurityHeadersMiddleware` (`X-Content-Type-Options`, `X-Frame-Options`, `X-XSS-Protection`, `Referrer-Policy`) em `monolith/src/http/middleware/security_headers.rs` e rate limiter (60 req/min) em `monolith/src/http/middleware/rate_limit.rs`
- [ ] T025 [US3] Implementar handler `POST /api/v1/vehicles/debts` com validação defensiva (`unknown_field` → 400, `invalid_plate` → 400, placa ausente/provider inválido → 400, `unknown_debt_type` → 422, `all_providers_unavailable` → 503) em `monolith/src/http/handlers/vehicle_debts.rs`
- [ ] T026 [P] [US3] Implementar handlers `GET /api/health` e `GET /api/health/integrations` (com cache em memória de 5s, bypass via `Cache-Control: no-cache` e header `X-Cache: HIT|MISS`) em `monolith/src/http/handlers/health.rs`
- [ ] T027 [P] [US3] Implementar handler `GET /api/metrics` em `monolith/src/http/handlers/metrics.rs`
- [ ] T028 [P] [US3] Implementar interface Web (`GET /` e consulta visual) com template Tailwind CSS em `monolith/templates/lookup.html` e `monolith/src/http/handlers/web_ui.rs`
- [ ] T029 [P] [US3] Implementar subcomando CLI `check-services` em `monolith/src/cli/check_services.rs`
- [ ] T030 [US3] Conectar `AppState`, rotas, middlewares e CLI em `monolith/src/http/routes.rs` e `monolith/src/main.rs`
- [ ] T031 [US3] Criar testes de integração da API HTTP, resiliência e cache em `monolith/tests/api_integration_test.rs`, `monolith/tests/fallback_resilience_test.rs` e `monolith/tests/observability_cache_test.rs`

---

## Phase 5: Dockerização Multi-Stage, Makefile e Validação Ponta a Ponta

**Purpose**: Integrar o monólito Rust com os containers PHP no Docker Compose e validar todos os gates da Constituição.

- [ ] T032 Criar `monolith/Dockerfile` multi-stage para compilação e execução do binário `infocar-monolith`
- [ ] T033 Criar `docker-compose.yml` e `Makefile` na raiz de `/Users/roger/Desktop/docker/infocar` conectando o `monolith` (Rust) aos serviços `provider-rest`, `provider-soap`, `provider-csv` e `payment-provider` (PHP)
- [ ] T034 Executar `make check` e `make demo` (`demo-success`, `demo-fallback`, `demo-invalid-plate`) validando 100% dos Gates de Qualidade da Constituição
