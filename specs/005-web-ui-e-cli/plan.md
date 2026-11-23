# Implementation Plan: Fase 5 — Web UI e CLI (`005-web-ui-e-cli`)

**Branch**: `005-web-ui-e-cli` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Módulos a Implementar

- `monolith/templates/lookup.html` (Askama + Tailwind CSS)
- `monolith/src/http/handlers/web_ui.rs`
- `monolith/src/cli/mod.rs` e `monolith/src/cli/check_services.rs`
- Integração em `monolith/src/http/routes.rs`, `monolith/src/lib.rs` e `monolith/src/main.rs`
