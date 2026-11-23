# API & Integration Contracts: Monólito Infocar (Rust) + Providers (PHP)

**Feature**: `001-rust-monolith-core`  
**Data**: 2026-11-23  

---

## 1. Contrato Público da API do Monólito (`porta 8000`)

### 1.1. `POST /api/v1/vehicles/debts`

#### Request Body (JSON)
```json
{
  "placa": "ABC1234",
  "provider": "rest"
}
```
- `"placa"` (obrigatório, string): trim + uppercase; deve satisfazer `^[A-Z]{3}[0-9][A-Z0-9][0-9]{2}$|^[A-Z]{3}[0-9]{4}$`.
- `"provider"` (opcional, string nullable): se presente, deve ser um de `"rest"`, `"soap"`, `"ai"`, `"csv"`.
- Qualquer outra propriedade no JSON é proibida.

#### Response `200 OK`
```json
{
  "placa": "ABC1234",
  "debitos": [
    {
      "tipo": "IPVA",
      "valor_original": "1500.00",
      "valor_atualizado": "1800.00",
      "vencimento": "2024-01-10",
      "dias_atraso": 121
    },
    {
      "tipo": "MULTA",
      "valor_original": "300.50",
      "valor_atualizado": "555.93",
      "vencimento": "2024-02-15",
      "dias_atraso": 85
    }
  ],
  "resumo": {
    "total_original": "1800.50",
    "total_atualizado": "2355.93"
  },
  "pagamentos": {
    "opcoes": [
      {
        "tipo": "TOTAL",
        "valor_base": "2355.93",
        "pix": {
          "total_com_desconto": "2238.13"
        },
        "cartao_credito": {
          "parcelas": [
            { "quantidade": 1, "valor_parcela": "2355.93" },
            { "quantidade": 6, "valor_parcela": "427.72" },
            { "quantidade": 12, "valor_parcela": "229.67" }
          ]
        }
      },
      {
        "tipo": "SOMENTE_IPVA",
        "valor_base": "1800.00",
        "pix": {
          "total_com_desconto": "1710.00"
        },
        "cartao_credito": {
          "parcelas": [
            { "quantidade": 1, "valor_parcela": "1800.00" },
            { "quantidade": 6, "valor_parcela": "326.79" },
            { "quantidade": 12, "valor_parcela": "175.48" }
          ]
        }
      },
      {
        "tipo": "SOMENTE_MULTA",
        "valor_base": "555.93",
        "pix": {
          "total_com_desconto": "528.13"
        },
        "cartao_credito": {
          "parcelas": [
            { "quantidade": 1, "valor_parcela": "555.93" },
            { "quantidade": 6, "valor_parcela": "100.93" },
            { "quantidade": 12, "valor_parcela": "54.20" }
          ]
        }
      }
    ]
  }
}
```

#### Respostas de Erro Padronizadas

| Cenário | HTTP Status | Body JSON Exato |
| :--- | :--- | :--- |
| Campos não reconhecidos no payload | `400 Bad Request` | `{"error": "unknown_field", "unrecognized_fields": ["foo"]}` |
| Placa enviada mas com formato inválido | `400 Bad Request` | `{"error": "invalid_plate"}` |
| Placa ausente ou vazia | `400 Bad Request` | `{"error": "A placa do veiculo e obrigatoria."}` |
| Provider informado inválido | `400 Bad Request` | `{"error": "O provedor informado e invalido. Provedores permitidos: rest, soap, ai, csv."}` |
| Tipo de débito desconhecido vindo de provider | `422 Unprocessable Entity` | `{"error": "unknown_debt_type", "type": "DPVAT"}` |
| Todos os provedores falharam | `503 Service Unavailable` | `{"error": "all_providers_unavailable"}` |

---

### 1.2. `GET /api/health`
- **Status**: `200 OK`
- **Body**:
  ```json
  {
    "status": "ok",
    "service": "monolith"
  }
  ```

### 1.3. `GET /api/health/integrations`
- **Headers de Resposta**: `X-Cache: HIT` ou `X-Cache: MISS`
- **Bypass**: Header de requisição `Cache-Control: no-cache` força `X-Cache: MISS`.
- **Status**: `200 OK` (quando todos os serviços estão `ok`) ou `503 Service Unavailable` (`"status": "degraded"`).
- **Body**:
  ```json
  {
    "status": "ok",
    "service": "monolith",
    "services": {
      "provider-rest": {
        "status": "ok",
        "url": "http://provider-rest:8000/api/health",
        "http_code": 200,
        "data": { "status": "ok", "service": "provider-rest" }
      },
      "provider-soap": {
        "status": "ok",
        "url": "http://provider-soap:8000/health",
        "http_code": 200,
        "data": { "status": "ok", "service": "provider-soap" }
      },
      "payment-provider": {
        "status": "ok",
        "url": "http://payment-provider:8000/health",
        "http_code": 200,
        "data": { "status": "ok", "service": "payment-provider" }
      }
    }
  }
  ```

### 1.4. `GET /api/metrics`
- **Status**: `200 OK`
- **Body**:
  ```json
  {
    "status": "ok",
    "metrics": {
      "vehicle_debt_requests_total": 0,
      "vehicle_debt_success_total": 0,
      "vehicle_debt_error_total": 0,
      "provider_requests_total": 0,
      "provider_failures_total": 0,
      "provider_retries_total": 0,
      "provider_fallbacks_total": 0
    }
  }
  ```

---

## 2. Contratos dos Eventos de Log Estruturado (JSON)

Cada linha emitida em `stdout` e em `monolith/storage/logs/laravel.log` DEVE ser um objeto JSON válido em linha única contendo `timestamp`, `level`, `message`, `request_id` e `event`:

1. **`request.received`**:
   ```json
   {"timestamp":"...","level":"INFO","message":"request.received","request_id":"...","event":"request.received","method":"POST","path":"api/v1/vehicles/debts"}
   ```
2. **`provider.request`**:
   ```json
   {"timestamp":"...","level":"INFO","message":"provider.request","request_id":"...","event":"provider.request","provider":"rest","operation":"get_debts","plate":"ABC****","attempt":1}
   ```
3. **`provider.response`**:
   ```json
   {"timestamp":"...","level":"INFO","message":"provider.response","request_id":"...","event":"provider.response","provider":"rest","status":"success","duration_ms":3,"debts_count":2}
   ```
4. **`vehicle_provider_failed`**:
   ```json
   {"timestamp":"...","level":"WARNING","message":"External vehicle debt provider failed","request_id":"...","event":"vehicle_provider_failed","provider":"rest","attempt":1,"error":"...","duration_ms":4,"plate":"ABC****","is_fatal_for_provider":false}
   ```
5. **`provider.retry`**:
   ```json
   {"timestamp":"...","level":"WARNING","message":"provider.retry","request_id":"...","event":"provider.retry","provider":"rest","attempt":2,"max_attempts":3,"reason":"...","backoff_ms":100}
   ```
6. **`provider.fallback`**:
   ```json
   {"timestamp":"...","level":"WARNING","message":"provider.fallback","request_id":"...","event":"provider.fallback","from":"rest","to":"soap","reason":"...","plate":"ABC****"}
   ```
7. **`vehicle_debt.completed`**:
   ```json
   {"timestamp":"...","level":"INFO","message":"vehicle_debt.completed","request_id":"...","event":"vehicle_debt.completed","plate":"ABC****","provider":"rest","debts_count":2,"duration_ms":5}
   ```
8. **`vehicle_debt.failed`**:
   ```json
   {"timestamp":"...","level":"WARNING","message":"vehicle_debt.failed","request_id":"...","event":"vehicle_debt.failed","plate":"INV****","error_type":"invalid_plate","status_code":400,"reason":"invalid_plate"}
   ```
