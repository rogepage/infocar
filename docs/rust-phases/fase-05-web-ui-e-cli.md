# Fase 5 — Interface Web (`GET /`) e Comando CLI de Diagnóstico

## Objetivo
Recriar a interface visual de consulta de débitos veiculares na rota raiz (`GET /`) e o utilitário de linha de comando para diagnóstico de conectividade com os containers satélites em PHP (`check-services`).

---

## 1. Interface Web (`GET /`)

O monólito original utiliza um componente Livewire (`VehicleDebtLookup`) com Tailwind CSS via CDN e fonte *Plus Jakarta Sans*. No monólito Rust:

- **Renderização**: Template `askama` (ou HTML estático + interação via formulário/Fetch/HTMX) servido em `GET /` (e `POST /` para submissão direta ou via `POST /api/v1/vehicles/debts`).
- **Estados Visuais Obrigatórios** (compatíveis com os testes de UI):
  1. **Formulário Inicial**:
     - Título `"Consulta de Débitos Veiculares"`
     - Input `placa` (`maxlength="7"`, `placeholder="ABC1234"`, uppercase)
     - Botão `"Consultar débitos"`
  2. **Mensagens de Erro Amigáveis**:
     - Placa vazia: `"A placa do veículo é obrigatória."`
     - Formato inválido: `"Placa inválida. Verifique o formato informado."`
     - `AllProvidersUnavailable` (503): `"Não foi possível consultar os débitos no momento. Tente novamente em alguns instantes."`
     - `UnknownDebtType` (422): `"O sistema encontrou um tipo de débito não reconhecido ({TIPO}). Entre em contato com o suporte."`
  3. **Estado de Veículo Sem Débitos (`debitos: []`)**:
     - Card verde exibindo `"Nenhum débito encontrado."` e `"Seu veículo está sem débitos disponíveis para consulta."`
  4. **Estado com Débitos e Opções de Pagamento**:
     - Cards de cada débito com badge do tipo (`IPVA`, `MULTA`, `LICENCIAMENTO`), dias em atraso, valor original formatado em `R$ 1.500,00`, vencimento em `DD/MM/YYYY` e valor atualizado.
     - Card escuro de **Resumo Consolidado** (`Total Original` e `Total Atualizado (com juros)`).
     - Cards de **Opções de Pagamento** (`Pagamento Total dos Débitos` e `Pagamento Parcial — <TIPO>`) mostrando o bloco **PIX à Vista (5% OFF)** e **Cartão de Crédito (1x, 6x, 12x)**.

---

## 2. Comando CLI de Diagnóstico (`check-services`)

Para manter total compatibilidade com `make health` e `make check`:

- O binário `infocar-monolith` deve aceitar o argumento CLI `check-services` (ex.: `infocar-monolith check-services`).
- Verifica a conectividade e tempo de resposta (em `ms`) de:
  - `Provider REST` (`{PROVIDER_REST_URL}/api/health`)
  - `Provider SOAP` (`{PROVIDER_SOAP_URL}/health`)
  - `Provider CSV/AI` (`{PROVIDER_CSV_URL}/api/health`)
  - `Payment Provider` (`{PAYMENT_PROVIDER_URL}/health`)
- Imprime no terminal uma tabela formatada contendo `Serviço | URL Alvo | Status (ONLINE/FAIL/UNREACHABLE) | HTTP Code | Latência | Resposta`.
- Retorna exit code `0` se todos estiverem `ONLINE` e `1` se houver qualquer falha.
