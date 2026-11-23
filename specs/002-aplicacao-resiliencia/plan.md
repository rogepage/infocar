# Implementation Plan: Fase 2 — Aplicação, Resiliência e Observabilidade (`002-aplicacao-resiliencia`)

**Branch**: `002-aplicacao-resiliencia` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Módulos a Implementar

- `monolith/src/application/plate_masker.rs`
- `monolith/src/infrastructure/observability/metrics.rs`
- `monolith/src/infrastructure/observability/logger.rs`
- `monolith/src/application/vehicle_debt/resolver.rs`
- `monolith/src/application/vehicle_debt/executor.rs`
- `monolith/src/application/vehicle_debt/service.rs`
- `monolith/src/application/vehicle_debt/result.rs`
- `monolith/src/application/vehicle_debt/use_case.rs`

## 2. Gates Verificados

- **Princípio III (Resiliência)**: Retry linear (`attempt * 100ms`) apenas para `Unavailable`, fail-fast para `InvalidResponse`, fallback *First-Success-Wins* e zero débitos sem fallback.
- **Princípio VI (Observabilidade e Privacidade)**: Mascaramento de placas via `PlateMasker` em todos os logs e JSON compatível com `scripts/format-demo-logs.py`.
