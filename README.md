# Infocar — Monólito em Rust + Provedores em PHP

> Reimplementação de alta performance do núcleo monolítico do **Cardok** em **Rust** (`axum` + `tokio`), preservando os serviços satélites (`provider-rest`, `provider-soap`, `provider-csv` e `payment-provider`) em **PHP** para comparação direta de arquitetura, latência, consumo de memória e resiliência entre as duas stacks.

---

## 1. Visão Geral e Objetivo do Projeto

O **Infocar** mantém **100% de paridade funcional e de contratos HTTP/JSON/Logs** com o projeto `cardok` original:
- **Monólito (`monolith/`)**: Escrito em **Rust**, estruturado com **Arquitetura Hexagonal (Ports & Adapters)** e **DDD**.
- **Provedores Externos (`provider-rest/`, `provider-soap/`, `provider-csv/`, `payment-provider/`)**: Mantidos em **PHP** em containers Docker independentes, simulando sistemas legados estaduais (Detran/Sefaz) com suporte a falhas controladas (`PROVIDER_MODE=success|error|timeout|invalid_response`).

---

## 2. Mapa da Documentação para Agentes e Desenvolvedores

Toda a documentação necessária para construir, validar e comparar o projeto está organizada abaixo:

### Governança e Instruções de Agentes
- [`AGENTS.md`](AGENTS.md) — Diretrizes operacionais para agentes de IA (Antigravity, Claude, Codex).
- [`CLAUDE.md`](CLAUDE.md) — Guia rápido de contexto para agentes.
- [`.specify/memory/constitution.md`](.specify/memory/constitution.md) — **Constituição do Projeto Infocar (v2.0.0)** com os 10 princípios e gates inegociáveis.

### Plano Arquitetural e Guias de Implementação em Rust (`docs/`)
- [`docs/00-plano-arquitetural-rust.md`](docs/00-plano-arquitetural-rust.md) — **Plano Arquitetural Completo** para recriar o monólito em Rust (mapeamento de classes PHP → módulos Rust, stack de crates, fórmulas financeiras exatas e roteiro de 7 fases).
- **Guias Passo a Passo por Fase (`docs/rust-phases/`)**:
  1. [`docs/rust-phases/fase-01-dominio-financeiro.md`](docs/rust-phases/fase-01-dominio-financeiro.md) — `Money(i64)`, `HalfUpRounder`, `Clock`, Políticas de Juros (IPVA, Multa, Licenciamento) e `PaymentSimulator` (PIX e Tabela Price).
  2. [`docs/rust-phases/fase-02-aplicacao-resiliencia.md`](docs/rust-phases/fase-02-aplicacao-resiliencia.md) — `PlateMasker`, `ProviderResolver`, `ProviderExecutor` (retry linear + fail-fast) e `VehicleDebtService` (*First-Success-Wins*).
  3. [`docs/rust-phases/fase-03-adapters-http-providers.md`](docs/rust-phases/fase-03-adapters-http-providers.md) — Clientes `reqwest` para os containers PHP (`RestVehicleDebtProvider`, `SoapVehicleDebtProvider`, `AiVehicleDebtProvider` em CSV defensivo).
  4. [`docs/rust-phases/fase-04-api-http-axum.md`](docs/rust-phases/fase-04-api-http-axum.md) — Rotas `axum`, validação defensiva (HTTP 400/422/503), cache de 5s com `X-Cache`, métricas atômicas e middlewares.
  5. [`docs/rust-phases/fase-05-web-ui-e-cli.md`](docs/rust-phases/fase-05-web-ui-e-cli.md) — Interface Web (`GET /`) com Tailwind CSS e subcomando CLI `check-services`.
  6. [`docs/rust-phases/fase-06-docker-e-makefile.md`](docs/rust-phases/fase-06-docker-e-makefile.md) — Multi-stage `Dockerfile` do monólito Rust, integração no `docker-compose.yml` com os providers PHP e `Makefile`.
  7. [`docs/rust-phases/fase-07-testes-e-comparacao.md`](docs/rust-phases/fase-07-testes-e-comparacao.md) — Suíte de testes em Rust (`cargo test`) e roteiro de benchmark comparativo `cardok` (PHP) vs `infocar` (Rust).

### Artefatos Spec-Driven Development (`specs/001-rust-monolith-core/`)
- [`specs/001-rust-monolith-core/spec.md`](specs/001-rust-monolith-core/spec.md) — Especificação completa da feature de recriação do monólito em Rust.
- [`specs/001-rust-monolith-core/plan.md`](specs/001-rust-monolith-core/plan.md) — Plano técnico de implementação.
- [`specs/001-rust-monolith-core/data-model.md`](specs/001-rust-monolith-core/data-model.md) — Modelo de dados canônico e tipagem em Rust.
- [`specs/001-rust-monolith-core/contracts/api-contracts.md`](specs/001-rust-monolith-core/contracts/api-contracts.md) — Contratos HTTP da API, dos provedores PHP e dos logs estruturados.
- [`specs/001-rust-monolith-core/tasks.md`](specs/001-rust-monolith-core/tasks.md) — Backlog de tarefas TDD (`T001` a `T040`) pronto para execução via `/speckit-implement`.

### Documentação Histórica de Referência (`docs/01-infra.md` .. `docs/14-readme.md`)
Os documentos originais de concepção do domínio do Cardok permanecem em `docs/` para consulta de requisitos históricos e casos de borda.

---

## 3. Como Continuar o Desenvolvimento com Agentes

Ao abrir `/Users/roger/Desktop/docker/infocar` no Antigravity IDE:

1. **Via Spec Kit (Recomendado)**:
   - Execute `/speckit-implement` para processar as tarefas ordenadas em [`specs/001-rust-monolith-core/tasks.md`](specs/001-rust-monolith-core/tasks.md); ou
   - Execute `/speckit-analyze` para auditar os artefatos de especificação e plano.
2. **Fase a Fase**:
   - Peça ao agente: *"Implemente a Fase 1 conforme `docs/rust-phases/fase-01-dominio-financeiro.md`"*, avançando incrementalmente até a Fase 7.
# infocar
