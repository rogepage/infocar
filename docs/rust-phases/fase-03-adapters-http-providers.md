# Fase 3 — Adaptadores HTTP para os Provedores Satélites em PHP

## Objetivo
Implementar em Rust os 3 adaptadores de infraestrutura (`RestVehicleDebtProvider`, `SoapVehicleDebtProvider` e `AiVehicleDebtProvider`) que consomem os containers satélites em PHP (`provider-rest`, `provider-soap` e `provider-csv`).

---

## 1. Módulos a Criar

```text
monolith/src/infrastructure/providers/
├── mod.rs
├── rest.rs      # RestVehicleDebtProvider (HTTP GET JSON)
├── soap.rs      # SoapVehicleDebtProvider (HTTP POST XML)
└── ai_csv.rs    # AiVehicleDebtProvider (HTTP GET Plaintext CSV Defensivo)
```

---

## 2. Contratos e Comportamento de Cada Adaptador

### 2.1. `RestVehicleDebtProvider` (`src/infrastructure/providers/rest.rs`)
- **Requisição**: `GET {base_url}/api/v1/vehicles/{plate}/debts`
- **Headers**: `X-Request-ID: {request_id}` (quando presente)
- **Timeout**: `PROVIDER_TIMEOUT` (padrão `2s`)
- **Tratamento de Resposta**:
  - Erro de conexão / timeout / status HTTP `500..=599` → `ProviderError::Unavailable`.
  - Status HTTP `!= 200` → `ProviderError::UnexpectedStatus`.
  - JSON inválido, ausência da chave `"debts"` (array), item sem `"type"`, `"amount"` ou `"due_date"`, valor monetário inválido ou data inválida → `ProviderError::InvalidResponse`.
  - Normaliza `amount` (aceitando número JSON como `1500.0` formatado para 2 casas decimais `"1500.00"`, inteiro ou string) via `Money::from_decimal`.
  - Retorna `ProviderDebtResponse { plate, debts, provider: "rest" }`.

### 2.2. `SoapVehicleDebtProvider` (`src/infrastructure/providers/soap.rs`)
- **Requisição**: `POST {base_url}/soap`
- **Headers**: `Content-Type: application/xml`, `X-Request-ID: {request_id}`
- **Payload**:
  ```xml
  <?xml version="1.0" encoding="UTF-8"?>
  <request>
      <plate>ABC1234</plate>
  </request>
  ```
- **Tratamento de Resposta**:
  - Erro de conexão / timeout / status HTTP `500..=599` → `ProviderError::Unavailable`.
  - Corpo vazio, XML malformado, raiz diferente de `<response>` ou ausência do elemento `<debts>` → `ProviderError::InvalidResponse`.
  - Se `<debts/>` não possuir elementos filhos `<debt>`, retorna `debts: vec![]` (veículo sem débitos).
  - Para cada `<debt>`, extrai `<category>`, `<value>` e `<expiration>`. Se algum estiver ausente, vazio ou com formato inválido → `ProviderError::InvalidResponse`.
  - Retorna `ProviderDebtResponse { plate, debts, provider: "soap" }`.

### 2.3. `AiVehicleDebtProvider` (`src/infrastructure/providers/ai_csv.rs`)
- **Requisição**: `GET {base_url}/api/v1/debts/{plate}`
- **Headers**: `Accept: text/plain, text/csv`, `X-Request-ID: {request_id}`
- **Regras Defensivas do Parser CSV**:
  1. Divide o corpo por quebras de linha (`\r\n`, `\r`, `\n`). Se vazio, retorna `vec![]`.
  2. Para cada linha (`trimmed`):
     - Ignora linhas vazias ou que comecem com ```` ``` ```` (cercas de código Markdown ```` ```csv ````).
     - Detecta o delimitador da linha: se `count(';') > count(',')` usa `';'`, senão `','`.
     - Faz o parse CSV da linha.
     - Se a primeira coluna (case-insensitive) for `"tipo"`, `"type"`, `"categoria"` ou `"category"`, ignora como linha de cabeçalho.
     - Se `columns.len() < 3`:
       - Se a linha for informativa (contiver termos como `"débito"`, `"debito"`, `"encontrado"`, `"aqui"`, `"veículo"`, `"veiculo"`, `"detran"`), ignora silenciosamente.
       - Caso contrário, incrementa `corrupt_lines_found` e continua.
     - Higieniza `raw_type`, `raw_amount` e `raw_date` removendo espaços e aspas.
     - Se `raw_type` não estiver em `["IPVA", "MULTA", "LICENCIAMENTO"]`:
       - Emite log `WARN` (`"Ignored unsupported debt type from AI provider"`, `raw_type`, `raw_line`) e ignora a linha.
     - Substitui `','` por `'.'` em `raw_amount`, converte com `Money::from_decimal` e `NaiveDate::parse_from_str`. Se falhar, incrementa `corrupt_lines_found`.
  3. Se ao final `corrupt_lines_found > 0 && valid_data_lines_found == 0 && debts.is_empty()`, retorna `Err(ProviderError::InvalidResponse)`.
  4. Retorna `ProviderDebtResponse { plate, debts, provider: "ai" }`.
