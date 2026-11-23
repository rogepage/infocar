<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/001-rust-monolith-core/plan.md
<!-- SPECKIT END -->

# Infocar — Diretrizes para Agentes de Engenharia (Monólito em Rust + Providers em PHP)

Você está trabalhando no projeto **Infocar**, uma reimplementação do núcleo monolítico do sistema **Cardok** em **Rust**, mantendo os serviços satélites (`provider-rest`, `provider-soap`, `provider-csv` e `payment-provider`) em **PHP** para permitir comparação direta entre as duas stacks sob os mesmos contratos de integração, resiliência e regras financeiras.

## 1. Documentos Obrigatórios de Leitura Inicial

Antes de planejar ou escrever código, leia:
1. **[Constituição do Projeto](.specify/memory/constitution.md)** — Princípios inegociáveis de arquitetura hexagonal, precisão financeira em centavos inteiros (`i64`) com arredondamento `HALF_UP`, resiliência *First-Success-Wins*, determinismo temporal e observabilidade.
2. **[Plano Arquitetural Geral Rust](docs/00-plano-arquitetural-rust.md)** — Análise completa do sistema original, mapeamento PHP → Rust, estrutura de módulos, fórmulas matemáticas exatas e plano em 7 fases.
3. **[Especificação da Feature Core](specs/001-rust-monolith-core/spec.md)** — Requisitos funcionais e critérios de aceite.
4. **[Plano Técnico Ativo](specs/001-rust-monolith-core/plan.md)** — Decisões de crates, árvore de arquivos e estratégia de build.
5. **[Modelo de Dados Canônico](specs/001-rust-monolith-core/data-model.md)** — Structs, Enums, Traits e invariantes de domínio.
6. **[Contratos de API e Logs](specs/001-rust-monolith-core/contracts/api-contracts.md)** — Payloads JSON/XML/CSV, códigos HTTP (200, 400, 422, 503), headers (`X-Request-ID`, `X-Cache`, Security Headers) e eventos de log estruturado.
7. **[Lista de Tarefas TDD](specs/001-rust-monolith-core/tasks.md)** — Tarefas ordenadas (`T001` a `T040`) para execução via `/speckit-implement`.

## 2. Regras Arquiteturais Inegociáveis

- **Domínio Puro (`monolith/src/domain/`)**: Zero dependências de frameworks web (`axum`), clientes HTTP (`reqwest`) ou formatos externos.
- **Precisão Financeira (`Money(i64)`)**:
  - Valores monetários representados exclusivamente em centavos inteiros (`i64`).
  - Proibido o uso de `f64` para armazenar ou somar montantes monetários.
  - `f64` é permitido **exclusivamente** para o coeficiente adimensional da Tabela Price ($k_n = \frac{i(1+i)^n}{(1+i)^n - 1}$ com $i = 0.025$), aplicando-o sobre os centavos inteiros e arredondando uma única vez com `HALF_UP`.
  - Arredondamento racional inteiro via `HalfUpRounder::round(numerator: i64, denominator: i64) -> i64`.
- **Determinismo Temporal (`Clock` trait)**:
  - A data de referência padrão do sistema é `2024-05-10T00:00:00Z` via `FixedClock`.
  - Proibido chamar `Utc::now()` diretamente dentro das regras de negócio do domínio.
- **Resiliência (*First-Success-Wins*)**:
  - Ordem configurável por `PROVIDER_ORDER` (padrão: `rest,soap,csv`).
  - Retries lineares (`attempt * 100ms`, padrão 2 retries = 3 tentativas totais) apenas para `ProviderUnavailable` (timeout, falha de conexão, HTTP 5xx).
  - *Fail-fast* imediato (sem retry no mesmo provider, acionando fallback) para `InvalidProviderResponse` (JSON/XML/CSV malformado).
  - Resposta válida com 0 débitos **NÃO** dispara fallback.
- **Privacidade e Observabilidade**:
  - Placas sempre mascaradas nos logs via `PlateMasker` (`ABC1234` → `ABC****`, `BRA2E19` → `BRA****`).
  - Propagação de `X-Request-ID` para todos os providers externos e retorno no response header.
  - Logs estruturados em JSON emitidos em `stdout` e espelhados em `monolith/storage/logs/laravel.log` para compatibilidade com scripts de demonstração (`scripts/format-demo-logs.py`).

## 3. Comandos de Referência

```bash
# Compilar e testar o monólito Rust localmente (ou via container)
cd monolith && cargo test

# Subir toda a stack Docker (Monólito Rust + Providers PHP)
make up

# Executar suíte completa de testes
make test

# Verificar saúde e conectividade com os providers PHP
make health

# Validação geral de conformidade (Gates da Constituição)
make check

# Executar cenários de demonstração (sucesso, fallback e placa inválida)
make demo
```
