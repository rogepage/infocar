# CLAUDE.md — Guia de Contexto do Projeto Infocar

Consulte [AGENTS.md](AGENTS.md) e [specs/001-rust-monolith-core/plan.md](specs/001-rust-monolith-core/plan.md) para todas as diretrizes arquiteturais, stack tecnológica em Rust, contratos de API e regras da Constituição do projeto.

## Resumo Rápido
- **Monólito (`monolith/`)**: Rust (Edition 2024 / 1.85+), `axum`, `tokio`, `reqwest`, `serde`, `chrono`, `roxmltree`, `askama`, `tracing`.
- **Provedores Satélites (`provider-rest/`, `provider-soap/`, `provider-csv/`, `payment-provider/`)**: Containers PHP independentes simulando sistemas legados externos via HTTP/JSON, SOAP/XML e Plaintext CSV.
- **Idioma dos Artefatos**: Português (pt-BR) para toda documentação, specs, planos, commits e comentários explicativos; identificadores de código em inglês (`Money`, `HalfUpRounder`, `DebtCalculationService`, `PaymentSimulator`, etc.).
