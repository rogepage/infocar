# Tasks: Fase 2 — Aplicação, Resiliência e Observabilidade (`002-aplicacao-resiliencia`)

- [x] T013 [P] Implementar `PlateMasker` e testes unitários em `monolith/src/application/plate_masker.rs`
- [x] T014 [P] Implementar `SimpleMetricsRegistry` (`AtomicU64`) e testes unitários em `monolith/src/infrastructure/observability/metrics.rs`
- [x] T015 [P] Implementar `StructuredLogger` (JSON `stdout` + arquivo) e testes unitários em `monolith/src/infrastructure/observability/logger.rs`
- [x] T016 Implementar `ProviderResolver` e testes unitários em `monolith/src/application/vehicle_debt/resolver.rs`
- [x] T017 Implementar `ProviderExecutor` (com `Sleeper` trait, retry linear e fail-fast) e testes unitários em `monolith/src/application/vehicle_debt/executor.rs`
- [x] T018 Implementar `VehicleDebtService` (*First-Success-Wins*), `VehicleDebtConsultationResult` e `GetVehicleDebtsUseCase` com testes unitários em `monolith/src/application/vehicle_debt/`
