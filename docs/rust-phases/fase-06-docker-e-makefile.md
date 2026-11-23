# Fase 6 — Dockerização Multi-Stage do Monólito Rust + Providers PHP

## Objetivo
Orquestrar no `docker-compose.yml` o container `monolith` compilado em **Rust** conversando na rede `cardok-network` (ou `infocar-network`) com os containers satélites em **PHP** (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`).

---

## 1. Reaproveitamento dos Providers em PHP

Os serviços satélites em PHP permanecem exatamente com a mesma implementação do projeto `cardok`:
- `provider-rest/` (porta host `8001` → interna `8000`)
- `provider-soap/` (porta host `8002` → interna `8000`)
- `provider-csv/` (porta host `8003` → interna `8000`)
- `payment-provider/` (porta interna `8000`)

Para copiar os providers PHP do projeto `cardok` para `infocar`:
```bash
cp -R /Users/roger/Desktop/docker/cardok/provider-rest /Users/roger/Desktop/docker/infocar/
cp -R /Users/roger/Desktop/docker/cardok/provider-soap /Users/roger/Desktop/docker/infocar/
cp -R /Users/roger/Desktop/docker/cardok/provider-csv /Users/roger/Desktop/docker/infocar/
cp -R /Users/roger/Desktop/docker/cardok/payment-provider /Users/roger/Desktop/docker/infocar/
cp -R /Users/roger/Desktop/docker/cardok/scripts /Users/roger/Desktop/docker/infocar/
```

---

## 2. `monolith/Dockerfile` (Rust Multi-Stage)

```dockerfile
# Stage 1: Build
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Cache de dependências
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Build do código-fonte real
COPY src ./src
COPY templates ./templates
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime enxuto
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /var/www/html
RUN mkdir -p /var/www/html/storage/logs

COPY --from=builder /app/target/release/infocar-monolith /usr/local/bin/infocar-monolith

EXPOSE 8000

CMD ["infocar-monolith"]
```

> **Dica de Compatibilidade com Logs de Demonstração:**  
> No `docker-compose.yml`, mapeie um volume `./monolith/storage/logs:/var/www/html/storage/logs` para que o arquivo `monolith/storage/logs/laravel.log` gravado pelo logger JSON do Rust fique acessível no host para o script `scripts/format-demo-logs.py` durante o `make demo-fallback`.

---

## 3. Ajustes no `Makefile`

No `Makefile` da raiz do `infocar`:
- `make test`: executa `cargo test` (localmente ou dentro do container builder/monolith).
- `make health`: executa `$(COMPOSE) exec $(MONOLITH_SERVICE) infocar-monolith check-services`.
- `make check`: valida `docker compose config`, sobe os containers com `--wait`, executa os testes do monólito Rust e roda o diagnóstico `check-services`.
- `make demo`: executa `demo-success`, `demo-fallback` e `demo-invalid-plate` sem nenhuma alteração nos comandos `curl`.
