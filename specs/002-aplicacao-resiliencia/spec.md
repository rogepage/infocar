# Feature Specification: Fase 2 — Camada de Aplicação, Resiliência e Observabilidade Base

**Feature Branch**: `002-aplicacao-resiliencia`  
**Phase**: Fase 2 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 2

Implementar a camada de orquestração (`GetVehicleDebtsUseCase`, `VehicleDebtService`, `ProviderExecutor`, `ProviderResolver`), o mascaramento LGPD de placas (`PlateMasker`), o registro atômico de métricas (`SimpleMetricsRegistry`) e o logger JSON estruturado (`StructuredLogger`) compatível com `stdout` e `storage/logs/laravel.log`.

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Mascaramento LGPD (`PlateMasker`) e Métricas Atômicas (`SimpleMetricsRegistry`) (Priority: P1)
1. **Given** uma placa `"abc1234"` ou `"BRA2E19"`, **When** processada por `PlateMasker::mask`, **Then** deve retornar `"ABC****"` e `"BRA****"`. Placas com menos de 4 caracteres devem retornar `"****"`.
2. **Given** chamadas concorrentes a `SimpleMetricsRegistry::increment`, **When** consultado via `get_all`, **Then** todos os 7 contadores atômicos devem refletir exatamente as contagens acumuladas.

### User Story 2 — Retry Linear e Fail-Fast (`ProviderExecutor`) (Priority: P1)
1. **Given** um provedor que retorna `ProviderError::Unavailable` nas tentativas 1 e 2 e sucesso na tentativa 3, **When** executado pelo `ProviderExecutor` (`max_retries = 2`, `initial_backoff_ms = 100`), **Then** deve executar 3 tentativas com backoffs de `100ms` e `200ms`, registrar `provider_retries_total = 2` e retornar `Ok`.
2. **Given** um provedor que retorna `ProviderError::InvalidResponse`, **When** executado pelo `ProviderExecutor`, **Then** deve fazer *fail-fast* na 1ª tentativa (sem retry) e retornar `Err`.

### User Story 3 — Fallback Sequencial First-Success-Wins (`VehicleDebtService` & `GetVehicleDebtsUseCase`) (Priority: P1)
1. **Given** ordem `rest,soap,csv` onde `rest` falha e `soap` responde com sucesso, **When** `VehicleDebtService::get_debts` for executado, **Then** deve incrementar `provider_fallbacks_total`, emitir log `provider.fallback` (`rest` → `soap`) e retornar os dados do `soap`.
2. **Given** um provedor que retorna `Ok` com `debts: []` (zero débitos), **When** `VehicleDebtService` processar, **Then** não deve acionar fallback para o próximo provedor.
3. **Given** todos os provedores falhando, **When** a lista se esgotar, **Then** deve retornar `Err(DomainError::AllProvidersUnavailable)`.
