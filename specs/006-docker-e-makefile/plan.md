# Implementation Plan: Fase 6 — Docker e Makefile (`006-docker-e-makefile`)

**Branch**: `006-docker-e-makefile` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

---

## 1. Artefatos da Fase 6

- `monolith/Dockerfile` (multi-stage `rust:1.88-slim-bookworm` → `debian:bookworm-slim`)
- `monolith/.dockerignore`
- `monolith/Cargo.lock`
- `docker-compose.yml` (monolith Rust + 4 providers PHP na rede `infocar-network`)
- `Makefile` (`up`, `down`, `test`, `health`, `check`, `demo`)
