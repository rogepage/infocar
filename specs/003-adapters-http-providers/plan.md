# Implementation Plan: Fase 3 — Adaptadores HTTP (`003-adapters-http-providers`)

**Branch**: `003-adapters-http-providers` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Módulos a Implementar

- `monolith/src/infrastructure/providers/mod.rs`
- `monolith/src/infrastructure/providers/rest.rs`
- `monolith/src/infrastructure/providers/soap.rs`
- `monolith/src/infrastructure/providers/ai_csv.rs`

## 2. Estratégia de Testes

Todos os adaptadores serão testados com servidor HTTP mock (`wiremock::MockServer`) validando:
- Payloads reais de sucesso e zero débitos
- Payloads malformados (`InvalidResponse`)
- HTTP 500 e Timeout (`Unavailable`)
- HTTP 404 (`UnexpectedStatus`)
- Propagação de `X-Request-ID`
