# Feature Specification: Fase 4 — Camada HTTP com Axum, Validação Defensiva e Middlewares

**Feature Branch**: `004-api-http-axum`  
**Phase**: Fase 4 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 4

Expor a API HTTP do monólito Rust na porta `8000` com `axum`, garantindo 100% de compatibilidade de rotas, payloads JSON, códigos HTTP (200, 400, 422, 429, 503), headers (`X-Request-ID`, `X-Cache: HIT|MISS`, Security Headers) e logs estruturados com o projeto `cardok`.

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Consulta de Débitos e Validação Defensiva (`POST /api/v1/vehicles/debts`) (Priority: P1)
1. **Given** payload válido `{"placa": "ABC1234"}`, **When** processado com sucesso, **Then** retorna HTTP 200 com `placa`, `debitos`, `resumo` e `pagamentos.opcoes`.
2. **Given** payload com campo extra (`{"placa": "ABC1234", "foo": "bar"}`), **Then** retorna HTTP 400 `{"error": "unknown_field", "unrecognized_fields": ["foo"]}`.
3. **Given** placa com formato inválido (`{"placa": "INVALID"}`), **Then** retorna HTTP 400 `{"error": "invalid_plate"}`.
4. **Given** tipo de débito desconhecido vindo de um provider, **Then** retorna HTTP 422 `{"error": "unknown_debt_type", "type": "..."}`.
5. **Given** todos os providers indisponíveis, **Then** retorna HTTP 503 `{"error": "all_providers_unavailable"}`.

### User Story 2 — Health Check, Cache de Integrações (5s) e Métricas (Priority: P1)
1. **Given** chamadas consecutivas a `GET /api/health/integrations` dentro de 5 segundos, **Then** a 1ª retorna `X-Cache: MISS` e a 2ª `X-Cache: HIT`.
2. **Given** header `Cache-Control: no-cache`, **Then** força `X-Cache: MISS` e reconsulta os serviços.
3. **Given** `GET /api/metrics`, **Then** retorna os 7 contadores atômicos em tempo real.

### User Story 3 — Middlewares Obrigatórios (`X-Request-ID`, Security Headers, Rate Limit) (Priority: P2)
1. **Given** qualquer requisição HTTP, **Then** todas as respostas incluem `X-Request-ID` e os 4 Security Headers (`X-Content-Type-Options`, `X-Frame-Options`, `X-XSS-Protection`, `Referrer-Policy`).
2. **Given** mais de 60 requisições em 1 minuto em `POST /api/v1/vehicles/debts`, **Then** retorna HTTP 429.
