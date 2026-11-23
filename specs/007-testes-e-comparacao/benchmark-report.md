# Benchmark Comparativo: Cardok (PHP/Laravel) vs Infocar (Rust/Axum)

**Data**: 2026-11-23  
**Condições**: Mesmo host macOS/Docker, mesma suíte Python (`ThreadPoolExecutor`), ambos consumindo os mesmos provedores satélites em PHP (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`).

---

## 1. Paridade de Contratos (`diff -u`)

Respostas JSON ordenadas de `POST /api/v1/vehicles/debts` comparadas entre `cardok` e `infocar`:
- `ABC1234`: **ZERO DIFF** (`0` diferenças)
- `LIC2024`: **ZERO DIFF** (`0` diferenças)

---

## 2. Footprint de Imagem e Memória

| Métrica | Cardok (PHP 8.4 / Laravel) | Infocar (Rust 1.88 / Axum) | Diferença |
| :--- | :---: | :---: | :---: |
| **Tamanho da Imagem Docker** | `176MB` | `148MB` | `-15.9%` |
| **Memória em Idle (`docker stats`)** | `73.08 MiB` | `2.47 MiB` | **~29,6x menor** |
| **Memória Pós-Carga (`docker stats`)** | `75.15 MiB` | `3.54 MiB` | **~21,2x menor** |

---

## 3. Throughput e Latência sob Concorrência

### 3.1. `GET /api/health` (500 requisições, concorrência 20 — baseline HTTP)

| Stack | Throughput | Tempo Total | p50 | p95 | p99 | Média |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Cardok (PHP)** | `49.58 req/s` | `10.084s` | `356.16 ms` | `618.51 ms` | `675.67 ms` | `396.56 ms` |
| **Infocar (Rust)** | **`1108.04 req/s`** | **`0.451s`** | **`15.72 ms`** | **`29.31 ms`** | **`36.67 ms`** | **`16.88 ms`** |
| **Ganho (Rust/PHP)** | **22,3x** | **22,4x mais rápido** | **-95,6%** | **-95,3%** | **-94,6%** | **-95,7%** |

### 3.2. `GET /api/health/integrations` (`X-Cache: HIT`, 300 requisições, concorrência 15)

| Stack | Throughput | Tempo Total | p50 | p95 | p99 | Média |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Cardok (PHP)** | `27.58 req/s` | `10.879s` | `340.92 ms` | `1947.34 ms` | `3088.94 ms` | `536.54 ms` |
| **Infocar (Rust)** | **`1295.67 req/s`** | **`0.232s`** | **`10.51 ms`** | **`17.82 ms`** | **`21.25 ms`** | **`11.25 ms`** |
| **Ganho (Rust/PHP)** | **47,0x** | **46,9x mais rápido** | **-96,9%** | **-99,1%** | **-99,3%** | **-97,9%** |

### 3.3. `POST /api/v1/vehicles/debts` (`ABC1234`, 50 requisições, concorrência 5 — ponta a ponta com `provider-rest` PHP)

| Stack | Throughput | Tempo Total | p50 | p95 | p99 | Média |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Cardok (PHP)** | `9.00 req/s` | `5.555s` | `532.08 ms` | `944.16 ms` | `1031.76 ms` | `535.47 ms` |
| **Infocar (Rust)** | **`36.01 req/s`** | **`1.388s`** | **`102.49 ms`** | **`434.58 ms`** | **`437.40 ms`** | **`135.15 ms`** |
| **Ganho (Rust/PHP)** | **4,0x** | **4,0x mais rápido** | **-80,7%** | **-54,0%** | **-57,6%** | **-74,8%** |

> **Observação**: No cenário 3.3, o gargalo principal passa a ser o serviço satélite `provider-rest` em PHP (`~100ms` por chamada sob concorrência); ainda assim, o monólito em Rust entrega **4x mais throughput** e **5,2x menor latência p50**.
