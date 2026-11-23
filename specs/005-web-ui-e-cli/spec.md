# Feature Specification: Fase 5 — Interface Web (`GET /`) e Diagnóstico CLI (`check-services`)

**Feature Branch**: `005-web-ui-e-cli`  
**Phase**: Fase 5 de 7  
**Created**: 2026-11-23  
**Status**: In Progress  

---

## 1. Objetivo da Fase 5

Recriar a interface web de consulta de débitos veiculares na rota raiz (`GET /` e `POST /`) usando template `askama` com Tailwind CSS e implementar o subcomando CLI `infocar-monolith check-services` para diagnóstico dos containers PHP satélites (`make health` e `make check`).

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Interface Web (`GET /` e `POST /`) (Priority: P2)
1. **Given** acesso a `GET /`, **Then** exibe o formulário `"Consulta de Débitos Veiculares"` com input `placa` (`maxlength="7"`, `placeholder="ABC1234"`) e botão `"Consultar débitos"`.
2. **Given** submissão de placa válida com débitos, **Then** exibe os cards de débito com tipo, dias em atraso, valor em `R$ 1.500,00`, vencimento `DD/MM/YYYY`, Resumo Consolidado e Opções de Pagamento (PIX 5% OFF + Cartão 1x, 6x, 12x).
3. **Given** placa sem débitos, **Then** exibe `"Nenhum débito encontrado."` e `"Seu veículo está sem débitos disponíveis para consulta."`.
4. **Given** erro de validação ou indisponibilidade, **Then** exibe mensagem amigável correspondente.

### User Story 2 — Diagnóstico CLI `check-services` (Priority: P1)
1. **Given** execução de `infocar-monolith check-services` com todos os 4 serviços PHP saudáveis, **Then** imprime tabela com status `ONLINE`, HTTP 200 e latência em `ms`, saindo com código `0`.
2. **Given** qualquer serviço offline/HTTP 5xx, **Then** marca `FAIL` ou `UNREACHABLE` e sai com código `1`.
