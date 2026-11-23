# Fase 4 — Camada HTTP com Axum, Validação Defensiva e Middlewares

## Objetivo
Expor a API HTTP do monólito Rust na porta `8000` com `axum`, garantindo 100% de compatibilidade de rotas, payloads, códigos de status HTTP, headers de segurança, `X-Request-ID`, `X-Cache` e rate limiting com o monólito PHP original.

---

## 1. Rotas e Contratos HTTP

| Método | Rota | Descrição |
| :--- | :--- | :--- |
| `POST` | `/api/v1/vehicles/debts` | Consulta débitos, calcula juros e simula pagamentos (PIX e Cartão) |
| `GET` | `/api/health` | Health check rápido do monólito (`{"status":"ok","service":"monolith"}`) |
| `GET` | `/api/health/integrations` | Diagnóstico dos providers externos com cache de 5s e header `X-Cache: HIT\|MISS` |
| `GET` | `/api/metrics` | Contadores operacionais atômicos em tempo real |
| `GET` | `/` | Interface web de consulta de débitos veiculares |

---

## 2. Validação Defensiva em `POST /api/v1/vehicles/debts`

Para manter paridade estrita com `VehicleDebtRequest.php`:

1. **Detecção de Campos Desconhecidos (Prioridade 1)**:
   - Deserializar o body JSON em `serde_json::Value`.
   - Verificar se há chaves no objeto JSON além de `"placa"` e `"provider"`.
   - Se houver, incrementar `vehicle_debt_error_total`, logar `vehicle_debt.failed` (`error_type: "unknown_field"`, `status_code: 400`) e retornar **HTTP 400**:
     ```json
     {
       "error": "unknown_field",
       "unrecognized_fields": ["campo_extra"]
     }
     ```
2. **Validação de Formato da Placa (Prioridade 0)**:
   - Se `"placa"` foi enviada (string não vazia após `trim`), converter para `UPPERCASE` e validar contra a regex:
     `^[A-Z]{3}[0-9][A-Z0-9][0-9]{2}$|^[A-Z]{3}[0-9]{4}$`
   - Se não corresponder, incrementar `vehicle_debt_error_total`, logar `vehicle_debt.failed` (`error_type: "invalid_plate"`, `status_code: 400`) e retornar **HTTP 400**:
     ```json
     {
       "error": "invalid_plate"
     }
     ```
3. **Placa Ausente ou Provider Inválido**:
   - Se `"placa"` estiver ausente, vazia ou não for string: retornar **HTTP 400** `{"error": "A placa do veiculo e obrigatoria."}`.
   - Se `"provider"` estiver presente (e não nulo/vazio) e não pertencer a `["rest", "soap", "ai", "csv"]`: retornar **HTTP 400** `{"error": "O provedor informado e invalido. Provedores permitidos: rest, soap, ai, csv."}`.
4. **Mapeamento de Exceções de Domínio/Aplicação**:
   - `DomainError::UnknownDebtType(debt_type)` → **HTTP 422**:
     ```json
     {
       "error": "unknown_debt_type",
       "type": "SEGURO_DPVAT"
     }
     ```
   - `DomainError::AllProvidersUnavailable` → **HTTP 503**:
     ```json
     {
       "error": "all_providers_unavailable"
     }
     ```

---

## 3. Cache de 5 Segundos em `GET /api/health/integrations`

- Se o header `Cache-Control` da requisição contiver `no-cache` (case-insensitive), faz bypass da leitura do cache.
- Caso contrário, se houver entrada em cache com idade `< 5 segundos`, retorna o payload e status code cacheados com o header `X-Cache: HIT`.
- Em caso de `MISS`:
  - Consulta sequencialmente ou em paralelo com timeout de `2s`:
    - `provider-rest`: `{rest_url}/api/health`
    - `provider-soap`: `{soap_url}/health`
    - `payment-provider`: `{payment_url}/health`
  - Se todos retornarem HTTP 2xx: `status_code = 200`, `"status": "ok"`.
  - Se qualquer um falhar: `status_code = 503`, `"status": "degraded"`.
  - Armazena no cache em memória por 5 segundos e retorna com header `X-Cache: MISS`.

---

## 4. Middlewares Obrigatórios

1. **`RequestIdMiddleware`**:
   - Lê o header `X-Request-ID`. Se corresponder a `^[a-zA-Z0-9\-_]{1,64}$`, preserva o valor; caso contrário, gera um `Uuid::new_v4().to_string()`.
   - Propaga o `request_id` no contexto da requisição (para logs e chamadas aos providers).
   - Se o path não for `api/health*` nem `up`, emite log `INFO` `request.received` (`method`, `path`).
   - Se o path contiver `vehicles/debts`, incrementa `vehicle_debt_requests_total`.
   - Anexa `X-Request-ID` nos headers da resposta.
2. **`SecurityHeadersMiddleware`**:
   - Anexa em todas as respostas:
     - `X-Content-Type-Options: nosniff`
     - `X-Frame-Options: SAMEORIGIN`
     - `X-XSS-Protection: 1; mode=block`
     - `Referrer-Policy: strict-origin-when-cross-origin`
3. **Rate Limiting (`60 req/min`)**:
   - Aplicado na rota `POST /api/v1/vehicles/debts`, retornando `HTTP 429 Too Many Requests` caso o limite de 60 requisições por minuto seja excedido.
