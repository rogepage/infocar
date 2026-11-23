# Feature Specification: Fase 7 — Suíte de Testes Completa e Benchmark do Monólito Rust

**Feature Branch**: `007-testes-e-comparacao`  
**Phase**: Fase 7 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 7

Completar a suíte de testes de integração (resiliência multi-provider via HTTP e interface Web UI) e coletar os indicadores reais de footprint (tamanho da imagem Docker, memória RSS em idle e latência de resposta) do monólito Rust (`infocar-monolith`) para comparação com a stack PHP (`cardok`).

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Testes de Resiliência Multi-Provider & Web UI (Priority: P1)
1. **Given** `rest` com JSON corrompido (*fail-fast*), `soap` com XML malformado (*fail-fast*) e `csv` saudável, **When** `POST /api/v1/vehicles/debts` for chamado, **Then** deve acionar fallback em cadeia `rest -> soap -> csv` sem retries e retornar HTTP 200 com os dados do `csv`.
2. **Given** `GET /` e `POST /` com placa válida, vazia, inválida e zero débitos, **Then** todos os estados visuais obrigatórios devem ser renderizados corretamente.

### User Story 2 — Medição de Footprint e Latência Real (Priority: P2)
1. **Given** a stack Docker em execução, **When** inspecionada com `docker images`, `docker stats --no-stream` e `curl`, **Then** os números observados devem ser documentados em `specs/007-testes-e-comparacao/benchmark-report.md`.
