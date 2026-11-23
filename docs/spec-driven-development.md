# Spec-Driven Development no Infocar (Monólito Rust + Providers PHP)

Este documento registra a configuração do **GitHub Spec Kit** no projeto **Infocar**, preparando o workspace para construir e evoluir o monólito em **Rust** através da metodologia **Spec-Driven Development (SDD)**.

---

## 1. Estrutura de Suporte

* **Constituição:** [`.specify/memory/constitution.md`](../.specify/memory/constitution.md) (v2.0.0 — adaptada para Rust no monólito e PHP nos providers satélites).
* **Skills do Agente:** `.agents/skills/speckit-*`
* **Regras de Agente:** [`AGENTS.md`](../AGENTS.md) e [`CLAUDE.md`](../CLAUDE.md)
* **Feature Principal Ativa:** [`specs/001-rust-monolith-core/`](../specs/001-rust-monolith-core/)
  * [`spec.md`](../specs/001-rust-monolith-core/spec.md)
  * [`plan.md`](../specs/001-rust-monolith-core/plan.md)
  * [`data-model.md`](../specs/001-rust-monolith-core/data-model.md)
  * [`contracts/api-contracts.md`](../specs/001-rust-monolith-core/contracts/api-contracts.md)
  * [`tasks.md`](../specs/001-rust-monolith-core/tasks.md)

---

## 2. Ciclo de Vida SDD

```text
Constitution (/speckit-constitution)
    │
    ▼
Specification (/speckit-specify)
    │
    ▼
Clarification (/speckit-clarify) [opcional]
    │
    ▼
Implementation Plan (/speckit-plan)
    │
    ▼
Task Breakdown (/speckit-tasks)
    │
    ▼
Consistency Analysis (/speckit-analyze) [opcional]
    │
    ▼
Implementation (/speckit-implement)
```
