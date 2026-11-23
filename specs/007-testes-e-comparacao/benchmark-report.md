# Relatório de Medição — Monólito Infocar (Rust / Axum)

**Data**: 2026-11-23  
**Branch**: `007-testes-e-comparacao`  

---

## 1. Footprint de Imagem e Memória (`docker images` / `docker stats --no-stream`)

| Container | Stack | Tamanho da Imagem | Uso de Memória (Observado) |
| :--- | :--- | :--- | :--- |
| `infocar-monolith` | Rust 1.88 (`axum` + `tokio`, `debian:bookworm-slim`) | `148MB` | `2.055MiB` (`0.03%`) |
| `infocar-provider-rest` | PHP 8.4 / Laravel | — | `59.93MiB` (`0.76%`) |
| `infocar-provider-soap` | PHP 8.4 / Laravel | — | `60.52MiB` (`0.76%`) |
| `infocar-provider-csv` | PHP 8.4 / Laravel | — | `59.04MiB` (`0.74%`) |
| `infocar-payment-provider` | PHP 8.4 CLI | — | `10.75MiB` (`0.14%`) |

---

## 2. Latência Observada (`curl`)

| Endpoint | Cenário | Tempo Total (`time_total`) |
| :--- | :--- | :--- |
| `GET /api/health/integrations` | Cache `X-Cache: HIT` (overhead puro Rust/Axum) | `0.002098s` (~`2.10 ms`) |
| `POST /api/v1/vehicles/debts` | Consulta completa `ABC1234` via `provider-rest` (PHP) | `0.040514s` (~`40.51 ms`) |

---

## 3. Como Comparar Lado a Lado com `cardok` (PHP)

1. Subir a stack do `cardok` (em `/Users/roger/Desktop/docker/cardok`) quando o `infocar` estiver parado (`make down`), ou em portas distintas.
2. Comparar:
   - `docker stats --no-stream` para o container `monolith`
   - `docker images` para a imagem do `monolith`
   - `diff -u cardok_abc1234.json infocar_abc1234.json` para validar paridade de contrato JSON.
