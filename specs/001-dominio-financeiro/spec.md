# Feature Specification: Fase 1 — Domínio Financeiro Puro e Simulação de Pagamentos (`monolith/src/domain/`)

**Feature Branch**: `001-dominio-financeiro`  
**Phase**: Fase 1 de 7  
**Created**: 2026-11-23  
**Status**: Implemented  

---

## 1. Objetivo da Fase 1

Implementar o núcleo de domínio financeiro do **Infocar** em Rust puro (`monolith/src/domain/`), totalmente isolado de frameworks web (`axum`), clientes HTTP (`reqwest`), XML, CSV ou infraestrutura, garantindo 100% de precisão centesimal (`i64`), arredondamento bancário `HALF_UP`, determinismo temporal (`FixedClock` em `2024-05-10T00:00:00Z`) e paridade ao centavo com o projeto original em PHP (`cardok`).

---

## 2. User Scenarios & Acceptance Criteria

### User Story 1 — Precisão Monetária `Money(i64)` e `HalfUpRounder` (Priority: P1)
Como engenheiro do domínio financeiro, preciso que todos os valores monetários sejam armazenados e manipulados exclusivamente em centavos inteiros (`i64`) e arredondados com `HALF_UP` racional sem uso de `f64`.

**Acceptance Scenarios**:
1. **Given** strings decimais válidas (`"1500.00"`, `"300.5"`, `"100"`, `"-10.05"`), **When** convertidas via `Money::from_decimal`, **Then** devem gerar `150000`, `30050`, `10000` e `-1005` centavos sem passar por `f64`.
2. **Given** strings inválidas (`"abc"`, `"12.345"`, `"12,50"`), **When** passadas a `Money::from_decimal`, **Then** devem retornar `Err(DomainError::InvalidMoneyFormat)`.
3. **Given** frações racionais nas fronteiras `.49`, `.50` e `.51` (positivas e negativas), **When** processadas por `HalfUpRounder::round`, **Then** `.50` deve arredondar para longe de zero (`150/100 -> 2`, `-150/100 -> -2`).

### User Story 2 — Políticas de Juros por Tipo de Débito (`IPVA`, `MULTA`, `LICENCIAMENTO`) (Priority: P1)
Como especialista de domínio, preciso que o cálculo de juros moratórios aplique a regra exata de cada tipo de débito usando a data de referência `2024-05-10`.

**Acceptance Scenarios**:
1. **Given** `IPVA` de `"1500.00"` vencido em `2024-01-10` (121 dias de atraso em `2024-05-10`), **When** calculado por `IpvaInterestPolicy`, **Then** a taxa de `0,33%/dia` (`39,93%`) deve ser limitada pelo teto de `20%` antes do arredondamento, resultando em juros de `"300.00"` e valor atualizado `"1800.00"`.
2. **Given** `MULTA` de `"300.50"` vencida em `2024-02-15` (85 dias de atraso), **When** calculada por `MultaInterestPolicy` (`1,00%/dia` sem teto), **Then** os juros devem ser `"255.43"` e o valor atualizado `"555.93"`.
3. **Given** `LICENCIAMENTO` de `"142.75"` com 40 dias de atraso, **When** calculado por `LicenciamentoInterestPolicy` (`0,33%/dia` com teto de `20%`), **Then** os juros devem ser `"18.84"` e o valor atualizado `"161.59"`.
4. **Given** um débito com categoria desconhecida (ex.: `"SEGURO_DPVAT"`), **When** processado por `DebtCalculationService`, **Then** deve retornar `Err(DomainError::UnknownDebtType("SEGURO_DPVAT"))`.

### User Story 3 — Simulação de Pagamentos (`PIX` 5% OFF e Cartão Tabela Price 1x, 6x, 12x) (Priority: P1)
Como consumidor do domínio, quero receber simulações de pagamento `TOTAL` e `SOMENTE_<TIPO>` (na ordem de primeira aparição dos débitos), com PIX (5% de desconto) e Cartão de Crédito (1x sem juros; 6x e 12x com Tabela Price a `2,5% a.m.`).

**Acceptance Scenarios**:
1. **Given** a placa `ABC1234` com `total_atualizado = "2355.93"` (`IPVA = "1800.00"`, `MULTA = "555.93"`), **When** `PaymentSimulator::simulate` for executado, **Then**:
   - `TOTAL`: PIX `"2238.13"`, Cartão 1x `"2355.93"`, 6x `"427.72"`, 12x `"229.67"`
   - `SOMENTE_IPVA`: PIX `"1710.00"`, Cartão 1x `"1800.00"`, 6x `"326.79"`, 12x `"175.48"`
   - `SOMENTE_MULTA`: PIX `"528.13"`, Cartão 1x `"555.93"`, 6x `"100.93"`, 12x `"54.20"`
2. **Given** um veículo sem débitos (`debts: []`), **When** simulado, **Then** `options` deve ser vazio (`[]`).
