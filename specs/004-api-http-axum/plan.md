# Implementation Plan: Fase 4 — API HTTP Axum (`004-api-http-axum`)

**Branch**: `004-api-http-axum` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Módulos a Implementar

- `monolith/src/http/state.rs` (`AppState`, `CachedHealthEntry`)
- `monolith/src/http/middleware/request_id.rs`
- `monolith/src/http/middleware/security_headers.rs`
- `monolith/src/http/middleware/rate_limit.rs`
- `monolith/src/http/handlers/vehicle_debts.rs`
- `monolith/src/http/handlers/health.rs`
- `monolith/src/http/handlers/metrics.rs`
- `monolith/src/http/routes.rs`
- Atualização de `monolith/src/lib.rs` e `monolith/src/main.rs`
