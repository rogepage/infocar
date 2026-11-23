# Tasks: Fase 4 — API HTTP Axum (`004-api-http-axum`)

- [x] T023 [P] Implementar `AppState` em `monolith/src/http/state.rs`
- [x] T024 [P] Implementar `RequestIdMiddleware`, `SecurityHeadersMiddleware` e `RateLimiter` em `monolith/src/http/middleware/`
- [x] T025 Implementar handler `POST /api/v1/vehicles/debts` com validação defensiva em `monolith/src/http/handlers/vehicle_debts.rs`
- [x] T026 [P] Implementar handlers `GET /api/health` e `GET /api/health/integrations` (cache 5s + `X-Cache`) em `monolith/src/http/handlers/health.rs`
- [x] T027 [P] Implementar handler `GET /api/metrics` em `monolith/src/http/handlers/metrics.rs`
- [x] T028 Conectar rotas e middlewares em `monolith/src/http/routes.rs`, `monolith/src/lib.rs` e `monolith/src/main.rs`
