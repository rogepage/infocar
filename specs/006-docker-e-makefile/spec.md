# Feature Specification: Fase 6 — Dockerização Multi-Stage e Orquestração Híbrida

**Feature Branch**: `006-docker-e-makefile`  
**Phase**: Fase 6 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 6

Integrar o monólito Rust (`monolith/Dockerfile` multi-stage com `rust:1.88-slim-bookworm` → `debian:bookworm-slim`) aos 4 serviços satélites em PHP (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`) via `docker-compose.yml` e `Makefile`, validando toda a stack com `make check` e `make demo`.

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Build Multi-Stage e Orquestração (`make check`) (Priority: P1)
1. **Given** `docker-compose.yml` e `monolith/Dockerfile`, **When** `make check` for executado, **Then** deve validar a sintaxe do Compose, subir os 5 containers saudáveis, rodar `make test` e executar `infocar-monolith check-services` com exit code `0`.

### User Story 2 — Cenários de Demonstração Ponta a Ponta (`make demo`) (Priority: P1)
1. **Given** a stack operacional, **When** `make demo` for executado, **Then**:
   - `demo-success` (`LIC2024`) deve retornar HTTP 200 com cálculo e parcelamento.
   - `demo-fallback` (`ABC1234` com REST em erro 500) deve registrar retries no REST, fallback para SOAP e retornar HTTP 200.
   - `demo-invalid-plate` (`INVALID`) deve retornar HTTP 400 `{"error":"invalid_plate"}`.
