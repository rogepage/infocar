# Feature Specification: Recriação do Monólito em Rust (Infocar) com Provedores Satélites em PHP

**Feature Branch**: `001-rust-monolith-core`  
**Created**: 2026-11-23  
**Status**: Approved  
**Input**: "Recriar o monólito do Cardok em Rust mantendo os provedores satélites em PHP para comparação direta entre as duas stacks."

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Domínio Financeiro Determinístico e Simulação de Pagamentos em Rust (Priority: P1) 🎯 MVP

Como engenheiro avaliando a stack Rust vs PHP, quero que o núcleo de domínio do monólito em Rust processe débitos canônicos (`IPVA`, `MULTA`, `LICENCIAMENTO`), calcule juros moratórios com data de referência fixada (`2024-05-10T00:00:00Z`) e gere simulações de pagamento (`TOTAL` e `SOMENTE_<TIPO>`, PIX com 5% de desconto e Cartão em 1x, 6x, 12x pela Tabela Price a 2,5% a.m.) operando estritamente em centavos inteiros (`i64`) com arredondamento `HALF_UP`.

**Why this priority**: A precisão financeira e a paridade ao centavo com o monólito PHP são requisitos inegociáveis da Constituição (Princípios I, II, IV e V).

**Independent Test**: Executar `cargo test` nos módulos `domain::debt` e `domain::payment` verificando que os valores calculados coincidem 100% com os vetores de teste do projeto `cardok`.

**Acceptance Scenarios**:

1. **Given** débitos de `IPVA` (`1500.00`, venc. `2024-01-10`) e `MULTA` (`300.50`, venc. `2024-02-15`) avaliados em `2024-05-10`,  
   **When** o `DebtCalculationService` e o `PaymentSimulator` forem executados,  
   **Then** o resultado deve apresentar `IPVA` atualizado `"1800.00"` (121 dias, teto de 20%), `MULTA` atualizada `"555.93"` (85 dias), `total_atualizado` `"2355.93"`, PIX `TOTAL` `"2238.13"` e parcelas de cartão `TOTAL` em 1x (`"2355.93"`), 6x (`"427.72"`) e 12x (`"229.67"`).

2. **Given** um débito de `LICENCIAMENTO` vencido,  
   **When** o cálculo de juros for executado,  
   **Then** deve aplicar `0,33%` ao dia limitado ao teto de `20%` antes do arredondamento `HALF_UP`, gerando a opção `SOMENTE_LICENCIAMENTO`.

3. **Given** um veículo sem débitos (`debts: []`),  
   **When** o domínio processar a resposta,  
   **Then** `total_original` e `total_atualizado` devem ser `"0.00"` e `pagamentos.opcoes` deve ser `[]`.

---

### User Story 2 - Integração HTTP com Provedores PHP (REST, SOAP, AI/CSV) e Resiliência First-Success-Wins (Priority: P1)

Como arquiteto de sistemas, quero que o monólito Rust consulte os containers satélites em PHP (`provider-rest`, `provider-soap`, `provider-csv`) normalizando JSON, XML e CSV defensivo para o modelo canônico, aplicando retry linear com backoff (`100ms`, `200ms`) para falhas transitórias, *fail-fast* para respostas malformadas e fallback sequencial *First-Success-Wins*.

**Why this priority**: Garante a interoperabilidade real entre o monólito Rust e os serviços legados simulados em PHP.

**Independent Test**: Simular falhas HTTP 500, timeouts e payloads corrompidos nos adaptadores Rust via `wiremock` e nos containers PHP reais, verificando retries, métricas e logs estruturados de fallback.

**Acceptance Scenarios**:

1. **Given** a ordem configurada `rest,soap,csv` e o `provider-rest` retornando HTTP 500,  
   **When** uma consulta for realizada,  
   **Then** o `ProviderExecutor` deve tentar o `rest` 3 vezes (1 inicial + 2 retries com backoff de 100ms e 200ms), acionar fallback para `soap` e retornar HTTP 200 com os dados do `soap`.

2. **Given** um provedor retornando payload malformado (JSON/XML/CSV inválido),  
   **When** o adaptador identificar a corrupção,  
   **Then** deve retornar `ProviderError::InvalidResponse`, abortando retries naquele provedor (*fail-fast*) e chaveando imediatamente para o próximo provedor.

3. **Given** uma resposta CSV do provedor de IA contendo blocos Markdown (```` ```csv ````), delimitadores `;` ou `,`, vírgula decimal e categorias não suportadas (`DPVAT`),  
   **When** o `AiVehicleDebtProvider` em Rust processar o texto,  
   **Then** deve extrair os débitos válidos, ignorar `DPVAT` emitindo log `WARN` e normalizar os valores sem uso de `float`.

---

### User Story 3 - API HTTP Compatível, Observabilidade, Cache de Health Check, UI Web e CLI (Priority: P2)

Como consumidor da API e avaliador técnico, quero que o monólito Rust exponha os mesmos endpoints HTTP (`POST /api/v1/vehicles/debts`, `GET /api/health`, `GET /api/health/integrations`, `GET /api/metrics`, `GET /`), os mesmos headers (`X-Request-ID`, `X-Cache`, Security Headers), logs estruturados em JSON com mascaramento de placas e o comando CLI `check-services`.

**Why this priority**: Permite executar exatamente os mesmos comandos do `Makefile` (`make check`, `make demo`, `make health`) e comparar `cardok` e `infocar` lado a lado sem alterar scripts clientes.

**Independent Test**: Executar `make check` e `make demo` contra a stack Docker do `infocar`.

**Acceptance Scenarios**:

1. **Given** um payload JSON com campos extras (`{"placa":"ABC1234","foo":"bar"}`) ou placa inválida (`{"placa":"INVALID"}`),  
   **When** enviado para `POST /api/v1/vehicles/debts`,  
   **Then** deve responder HTTP 400 com `{"error":"unknown_field","unrecognized_fields":["foo"]}` ou `{"error":"invalid_plate"}`.

2. **Given** chamadas consecutivas a `GET /api/health/integrations` dentro de 5 segundos,  
   **When** a primeira e a segunda chamadas forem inspecionadas,  
   **Then** a 1ª deve retornar `X-Cache: MISS` e a 2ª `X-Cache: HIT` (a menos que `Cache-Control: no-cache` seja enviado).

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O monólito Rust DEVE implementar `Money(i64)` e `HalfUpRounder` sem uso de ponto flutuante para valores monetários.
- **FR-002**: O monólito Rust DEVE calcular juros de `IPVA` (0,33%/dia, teto 20%), `LICENCIAMENTO` (0,33%/dia, teto 20%) e `MULTA` (1,00%/dia, sem teto) usando `Clock` injetável fixado em `2024-05-10T00:00:00Z`.
- **FR-003**: O monólito Rust DEVE simular pagamentos `TOTAL` e `SOMENTE_<TIPO>` com PIX (5% de desconto) e Cartão de Crédito (1x sem juros; 6x e 12x via Tabela Price a 2,5% a.m.).
- **FR-004**: O monólito Rust DEVE implementar adaptadores HTTP (`RestVehicleDebtProvider`, `SoapVehicleDebtProvider`, `AiVehicleDebtProvider`) consumindo os containers PHP satélites com timeout configurável (`PROVIDER_TIMEOUT`, padrão `2s`) e propagação de `X-Request-ID`.
- **FR-005**: O monólito Rust DEVE implementar `ProviderExecutor` (retry linear `attempt * 100ms` para `Unavailable` e fail-fast para `InvalidResponse`) e `VehicleDebtService` (fallback *First-Success-Wins* respeitando `PROVIDER_ORDER` ou override `"provider"` no payload).
- **FR-006**: O monólito Rust DEVE expor `POST /api/v1/vehicles/debts` (com rate limit de 60 req/min), `GET /api/health`, `GET /api/health/integrations` (cache 5s + `X-Cache`), `GET /api/metrics` e `GET /` (interface web Tailwind).
- **FR-007**: O monólito Rust DEVE mascarar placas nos logs (`ABC1234` → `ABC****`), emitir logs JSON estruturados em `stdout` e `storage/logs/laravel.log`, e anexar headers HTTP de segurança em todas as respostas.
- **FR-008**: O binário do monólito Rust DEVE disponibilizar o subcomando CLI `check-services` para diagnóstico de conectividade com os containers PHP.

---

## Success Criteria *(mandatory)*

- **SC-001**: 100% de equivalência nos payloads JSON e códigos HTTP (200, 400, 422, 503) entre o monólito Rust (`infocar`) e o monólito PHP (`cardok`).
- **SC-002**: 100% dos testes unitários e de integração em Rust (`cargo test`) passando sem warnings.
- **SC-003**: `make check` e `make demo` (`demo-success`, `demo-fallback`, `demo-invalid-plate`) executando com sucesso ponta a ponta com o monólito em Rust e os providers em PHP.
