# Fase 7 — Suíte de Testes em Rust e Roteiro de Comparação (PHP vs Rust)

## Objetivo
Garantir 100% de cobertura dos cenários do `cardok` na suíte `cargo test` do `infocar` e fornecer uma metodologia reprodutível para comparar as duas stacks (Monólito PHP/Laravel vs Monólito Rust/Axum, ambos consumindo os mesmos providers PHP).

---

## 1. Matriz de Testes Obrigatórios em Rust (`cargo test`)

| Categoria | Módulos Testados | Cenários Obrigatórios |
| :--- | :--- | :--- |
| **Domínio — Money & Rounding** | `domain::debt::money`, `domain::debt::rounding` | Criação por centavos e decimal, rejeição de formato inválido, soma/subtração, `HalfUpRounder` em fronteiras `.49`, `.50`, `.51` e negativos. |
| **Domínio — Políticas de Juros** | `domain::debt::policies`, `calculation_service` | IPVA (0,33%/dia + teto 20% antes do round), Licenciamento (0,33%/dia + teto 20%), Multa (1%/dia sem teto), débito não vencido (`dias_atraso = 0` → juros `0.00`), erro `UnknownDebtType`. |
| **Domínio — Simulador de Pagamentos** | `domain::payment::*` | PIX com 5% de desconto (`HALF_UP`), Cartão 1x, 6x e 12x (Tabela Price 2,5% a.m.), agrupamento `TOTAL` e `SOMENTE_<TIPO>`, veículo sem débitos (`opcoes: []`). |
| **Adapters — REST, SOAP e AI/CSV** | `infrastructure::providers::*` | Parsing JSON válido/corrompido, SOAP XML com `<debts>` e `<debts/>` auto-fechada, CSV com `,` e `;`, blocos Markdown ```` ```csv ````, comentários, descarte de `DPVAT` com warning, falha em CSV 100% corrompido, HTTP 500 e timeout. |
| **Aplicação — Retry e Fallback** | `application::vehicle_debt::*` | Retry linear (`100ms`, `200ms`) em `Unavailable`, fail-fast sem retry em `InvalidResponse`, fallback *First-Success-Wins* (`rest -> soap -> csv`), 0 débitos não dispara fallback, exaustão total (`AllProvidersUnavailable`). |
| **API HTTP & Observabilidade** | `http::*` | `POST /api/v1/vehicles/debts` (200, 400 `invalid_plate`, 400 `unknown_field`, 422 `unknown_debt_type`, 503 `all_providers_unavailable`), `GET /api/health/integrations` (`X-Cache: MISS` → `HIT` + `Cache-Control: no-cache`), `GET /api/metrics`, `X-Request-ID`, Security Headers. |

---

## 2. Roteiro de Benchmark Comparativo (`cardok` vs `infocar`)

Com as duas pastas separadas (`/Users/roger/Desktop/docker/cardok` e `/Users/roger/Desktop/docker/infocar`), você pode comparar as duas stacks sob condições idênticas:

### Dimensões de Comparação

1. **Consumo de Memória em Idle e Sob Carga (`docker stats --no-stream`)**:
   - Medir RSS do container `monolith` em PHP/Laravel vs Rust/Axum.
2. **Tamanho da Imagem Docker (`docker images`)**:
   - Comparar o tamanho final da imagem do `monolith` em PHP vs multi-stage `debian-slim` / `alpine` em Rust.
3. **Latência Interna de Orquestração e Cálculo (`duration_ms` nos logs e `/api/health/integrations` em `X-Cache: HIT`)**:
   - Testar o endpoint cacheado `/api/health/integrations` (que mede puramente o overhead do framework HTTP + memória sem I/O dos providers PHP).
   - Testar `POST /api/v1/vehicles/debts` sob concorrência (ex.: `oha`, `wrk` ou `ab -n 1000 -c 20`).
4. **Precisão e Paridade de Contratos (`diff`)**:
   - Salvar o JSON de resposta de `POST /api/v1/vehicles/debts` para as placas `ABC1234`, `LIC2024`, `ZERO000` em ambas as stacks e executar `diff -u cardok.json infocar.json` — a saída deve ser **zero diferenças**.
