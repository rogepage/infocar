# Feature Specification: Fase 3 — Adaptadores HTTP para os Provedores Satélites em PHP

**Feature Branch**: `003-adapters-http-providers`  
**Phase**: Fase 3 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 3

Implementar os 3 adaptadores de infraestrutura (`RestVehicleDebtProvider`, `SoapVehicleDebtProvider` e `AiVehicleDebtProvider`) em `monolith/src/infrastructure/providers/` consumindo os serviços PHP satélites (`provider-rest`, `provider-soap`, `provider-csv`) com normalização para o modelo canônico `ProviderDebtResponse`, propagação de `X-Request-ID`, timeout configurável e tratamento rigoroso de erros (`Unavailable` vs `InvalidResponse` vs `UnexpectedStatus`).

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — `RestVehicleDebtProvider` (JSON) (Priority: P1)
1. **Given** resposta HTTP 200 JSON válida com array `"debts"`, **When** processada pelo adaptador REST, **Then** deve normalizar `amount` (string ou número) via `Money::from_decimal` e retornar `ProviderDebtResponse { provider: "rest", .. }`.
2. **Given** JSON malformado, ausência de `"debts"` ou campo obrigatório inválido, **When** processado, **Then** deve retornar `ProviderError::InvalidResponse` (*fail-fast*).
3. **Given** HTTP 5xx ou timeout, **Then** deve retornar `ProviderError::Unavailable`.

### User Story 2 — `SoapVehicleDebtProvider` (XML) (Priority: P1)
1. **Given** resposta HTTP 200 XML `<response><debts><debt>...</debt></debts></response>`, **When** processada via `roxmltree`, **Then** deve retornar `ProviderDebtResponse { provider: "soap", .. }`.
2. **Given** tag `<debts/>` vazia (sem filhos `<debt>`), **Then** deve retornar `Ok` com `debts: []`.
3. **Given** XML malformado, raiz diferente de `<response>` ou tag obrigatória ausente, **Then** deve retornar `ProviderError::InvalidResponse`.

### User Story 3 — `AiVehicleDebtProvider` (CSV Defensivo) (Priority: P1)
1. **Given** CSV com cercas Markdown ```` ```csv ````, delimitadores `;` ou `,`, vírgula decimal (`"1500,00"`) e linhas informativas, **When** processado, **Then** deve extrair os débitos válidos sem usar `f64`.
2. **Given** categoria não suportada (ex.: `DPVAT`), **Then** deve emitir log `WARNING` e ignorar a linha.
3. **Given** CSV 100% corrompido (sem nenhuma linha válida), **Then** deve retornar `ProviderError::InvalidResponse`.
