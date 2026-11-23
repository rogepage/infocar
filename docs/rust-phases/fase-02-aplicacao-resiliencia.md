# Fase 2 — Camada de Aplicação, Resiliência e Observabilidade Base

## Objetivo
Implementar a orquestração da consulta de débitos (`GetVehicleDebtsUseCase`, `VehicleDebtService`, `ProviderExecutor`, `ProviderResolver`), o mascaramento LGPD de placas (`PlateMasker`) e o registro atômico de métricas (`SimpleMetricsRegistry`).

---

## 1. Módulos a Criar

```text
monolith/src/
├── application/
│   ├── mod.rs
│   ├── plate_masker.rs              # PlateMasker::mask("ABC1234") -> "ABC****"
│   └── vehicle_debt/
│       ├── mod.rs
│       ├── resolver.rs              # ProviderResolver
│       ├── executor.rs              # ProviderExecutor (retry linear + backoff + Sleeper injetável)
│       ├── service.rs               # VehicleDebtService (fallback First-Success-Wins)
│       ├── result.rs                # VehicleDebtConsultationResult
│       └── use_case.rs              # GetVehicleDebtsUseCase
└── infrastructure/
    └── observability/
        ├── mod.rs
        ├── metrics.rs               # SimpleMetricsRegistry (AtomicU64)
        └── logger.rs                # Emissão de eventos JSON estruturados
```

---

## 2. Regras de Implementação

### 2.1. `PlateMasker` (`src/application/plate_masker.rs`)
- Higieniza a placa (`trim().to_uppercase()`).
- Se `len < 4`, retorna `"****"`.
- Caso contrário, mantém os primeiros `len - 4` caracteres e concatena `"****"` (ex.: `"ABC1234"` → `"ABC****"`, `"BRA2E19"` → `"BRA****"`).

### 2.2. `SimpleMetricsRegistry` (`src/infrastructure/observability/metrics.rs`)
- Mantém 7 contadores `AtomicU64`:
  1. `vehicle_debt_requests_total`
  2. `vehicle_debt_success_total`
  3. `vehicle_debt_error_total`
  4. `provider_requests_total`
  5. `provider_failures_total`
  6. `provider_retries_total`
  7. `provider_fallbacks_total`
- Métodos: `increment(metric)`, `get_all() -> BTreeMap/IndexMap`, `reset()`.

### 2.3. `ProviderResolver` (`src/application/vehicle_debt/resolver.rs`)
- Mantém mapa `HashMap<String, Arc<dyn VehicleDebtProvider>>` com as chaves `"rest"`, `"soap"`, `"ai"`, `"csv"` (onde `"ai"` e `"csv"` apontam para o adaptador CSV/AI).
- `get_configured_order()` retorna a lista configurada por `PROVIDER_ORDER` (padrão `["rest", "soap", "csv"]`) filtrada pelas chaves registradas.

### 2.4. `ProviderExecutor` (`src/application/vehicle_debt/executor.rs`)
- Configurado com `max_retries: u32` (padrão `2`, total de `3` tentativas) e `initial_backoff_ms: u64` (padrão `100`).
- Em cada tentativa (`attempt = 1..=total_attempts`):
  1. Emite log `INFO` `provider.request` (`request_id`, `provider`, `operation: "get_debts"`, `plate: masked`, `attempt`) e incrementa `provider_requests_total`.
  2. Chama `provider.get_debts(plate, request_id).await`.
  3. **Sucesso**: emite log `INFO` `provider.response` (`status: "success"`, `duration_ms`, `debts_count`) e retorna `Ok(response)`.
  4. **`ProviderError::InvalidResponse` (Não-retriable / Fail-fast)**:
     - Incrementa `provider_failures_total`.
     - Emite log `WARN` `vehicle_provider_failed` com `is_fatal_for_provider: true`.
     - Retorna `Err` imediatamente sem executar novas tentativas neste provider.
  5. **`ProviderError::Unavailable` (Retriable)**:
     - Incrementa `provider_failures_total`.
     - Emite log `WARN` `vehicle_provider_failed` com `is_fatal_for_provider: attempt == total_attempts`.
     - Se `attempt < total_attempts`:
       - Calcula `backoff_ms = (attempt as u64) * initial_backoff_ms`.
       - Emite log `WARN` `provider.retry` (`attempt: attempt + 1`, `max_attempts`, `reason`, `backoff_ms`).
       - Incrementa `provider_retries_total`.
       - Aguarda `sleeper.sleep(backoff_ms).await`.

### 2.5. `VehicleDebtService` (`src/application/vehicle_debt/service.rs`)
- Itera sequencialmente pelos providers da ordem ativa (`custom_order` ou `configured_order`).
- **First-Success-Wins**: o primeiro provider que retornar `Ok(ProviderDebtResponse)` (mesmo com `debts: []`) encerra a busca e retorna o resultado.
- Quando um provider falha e existe um `next_provider`:
  - Incrementa `provider_fallbacks_total`.
  - Emite log `WARN` `provider.fallback` (`from`, `to`, `reason`, `plate: masked`).
- Se todos falharem:
  - Emite log `ERROR` `all_vehicle_providers_failed` (`plate: masked`, `tried_providers`, `errors`).
  - Retorna `Err(DomainError::AllProvidersUnavailable)`.
