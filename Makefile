# ==============================================================================
# Infocar (Monólito Rust + Providers PHP) — Makefile de Operação e Validação
# ==============================================================================

.DEFAULT_GOAL := help
.SHELL := /bin/sh

COMPOSE ?= docker compose
MONOLITH_SERVICE ?= monolith

.PHONY: help build up down restart status logs test health check shell demo demo-success demo-fallback demo-invalid-plate

help: ## Mostra esta lista de comandos disponíveis
	@echo "Infocar (Rust Monolith + PHP Providers) — Comandos disponíveis:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-20s %s\n", $$1, $$2}'
	@echo ""

# ------------------------------------------------------------------------------
# Ciclo de Vida do Ambiente Docker
# ------------------------------------------------------------------------------

build: ## Compila as imagens Docker (incluindo multi-stage Rust do monólito)
	@echo "==> Compilando imagens Docker do Infocar..."
	@$(COMPOSE) build

up: ## Inicia todos os serviços em background e aguarda prontidão
	@echo "==> Iniciando ambiente Infocar e aguardando serviços saudáveis..."
	@$(COMPOSE) up -d --build --wait
	@echo "==> Ambiente operacional e saudável. Execute 'make status' para acompanhar os serviços."

down: ## Para todos os serviços (preserva volumes de dados)
	@echo "==> Encerrando ambiente Infocar..."
	@$(COMPOSE) down

restart: ## Reinicia todos os serviços
	@echo "==> Reiniciando serviços do Infocar..."
	@$(COMPOSE) restart

status: ## Mostra o status e saúde dos contêineres
	@echo "==> Verificando status dos serviços..."
	@$(COMPOSE) ps

logs: ## Acompanha os logs unificados em tempo real
	@$(COMPOSE) logs -f

# ------------------------------------------------------------------------------
# Testes, Saúde e Validação (CI/CD)
# ------------------------------------------------------------------------------

test: ## Executa a suíte de testes automatizados em Rust (cargo test)
	@echo "==> Executando testes automatizados do monólito Rust..."
	@if command -v cargo >/dev/null 2>&1; then \
		cd monolith && cargo test; \
	else \
		docker run --rm -v "$$(pwd)/monolith":/app -w /app rust:1.88-slim-bookworm cargo test; \
	fi

health: ## Executa diagnóstico de conectividade com serviços externos (PHP providers)
	@echo "==> Verificando saúde e conectividade das integrações..."
	@$(COMPOSE) exec $(MONOLITH_SERVICE) infocar-monolith check-services

check: ## Executa validação geral de conformidade (config, status, testes, saúde)
	@echo "==> [1/4] Validando sintaxe do Docker Compose..."
	@$(COMPOSE) config --quiet
	@echo "    ✓ Configuração do Docker Compose válida."
	@echo "==> [2/4] Verificando status dos contêineres e aguardando prontidão..."
	@$(COMPOSE) up -d --build --wait
	@$(COMPOSE) ps
	@echo "==> [3/4] Executando suíte completa de testes em Rust..."
	@$(MAKE) test
	@echo "==> [4/4] Executando diagnóstico de integrações..."
	@$(COMPOSE) exec $(MONOLITH_SERVICE) infocar-monolith check-services
	@echo ""
	@echo "==> ✓ Todas as validações foram concluídas com sucesso!"

# ------------------------------------------------------------------------------
# Acesso Operacional
# ------------------------------------------------------------------------------

shell: ## Abre um shell interativo no contêiner do monólito Rust
	@echo "==> Abrindo shell no monólito..."
	@$(COMPOSE) exec $(MONOLITH_SERVICE) sh

# ------------------------------------------------------------------------------
# Demonstrações Técnicas
# ------------------------------------------------------------------------------

demo-success: ## Demonstração: consulta débitos com sucesso (placa LIC2024)
	@echo "==> [DEMO] Consultando débitos da placa LIC2024 (Sucesso com cálculo e parcelamento):"
	@curl -s -X POST http://localhost:8000/api/v1/vehicles/debts \
		-H "Content-Type: application/json" \
		-d '{"placa":"LIC2024"}' | (command -v jq >/dev/null 2>&1 && jq . || cat)
	@echo ""

demo-fallback: ## Demonstração: fallback real com falha controlada no REST, retries e sucesso no SOAP
	@./scripts/demo-fallback.sh

demo-invalid-plate: ## Demonstração: validação com formato de placa inválido
	@echo "==> [DEMO] Consultando com placa inválida INVALID (Validação e erro 400):"
	@curl -s -X POST http://localhost:8000/api/v1/vehicles/debts \
		-H "Content-Type: application/json" \
		-d '{"placa":"INVALID"}' | (command -v jq >/dev/null 2>&1 && jq . || cat)
	@echo ""

demo: demo-success demo-fallback demo-invalid-plate ## Executa todos os cenários de demonstração em sequência
	@echo "==> ✓ Todos os cenários de demonstração foram executados!"
