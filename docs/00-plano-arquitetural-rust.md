# Plano Arquitetural Completo: Recriação do Monólito em Rust (`Infocar`)

**Projeto:** Infocar (Comparativo de Stack com `Cardok`)  
**Escopo:** Monólito principal (`monolith/`) 100% em **Rust**; provedores satélites (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`) mantidos em **PHP**.  
**Data:** 2026-11-23  

---

## 1. Análise do Sistema Original e Mapeamento PHP → Rust

O monólito original em PHP 8.4 / Laravel 10 foi desenhado sob **Arquitetura Hexagonal (Ports & Adapters)** e **Domain-Driven Design (DDD)**, operando de forma *stateless* (sem dependência de banco de dados relacional em tempo de execução, utilizando apenas memória/cache para métricas e healthcheck).

### Tabela de Equivalência Arquitetural

| Camada / Conceito | Implementação Original (PHP / Laravel) | Equivalente em Rust (`monolith/`) |
| :--- | :--- | :--- |
| **Runtime & Servidor HTTP** | PHP 8.4 CLI (`php artisan serve`) + Laravel Router | `tokio` (async runtime) + `axum` (HTTP server na porta `8000`) |
| **Injeção de Dependências** | Service Container do Laravel (`AppServiceProvider`) | `AppState` (`Arc<AppContext>`) compartilhado via `axum::extract::State` |
| **Value Object Monetário** | `App\Domain\Debt\Money` (`readonly class`, `int $amountInCents`) | `pub struct Money(i64)` (`#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]`) |
| **Arredondamento Bancário** | `HalfUpRounder::round(int $num, int $den): int` (`intdiv`) | `HalfUpRounder::round(numerator: i64, denominator: i64) -> Result<i64, DomainError>` |
| **Abstração de Tempo** | `ClockInterface` + `FixedClock` (`CarbonImmutable`) | `pub trait Clock: Send + Sync` + `FixedClock` (`chrono::DateTime<Utc>`) |
| **Políticas de Juros** | `DebtInterestPolicyInterface` + `Ipva`, `Multa`, `Licenciamento` | `pub trait DebtInterestPolicy: Send + Sync` + structs concretas + `DebtInterestPolicyRegistry` |
| **Cálculo Price e PIX** | `PixCalculator`, `CreditCardCalculator`, `PaymentSimulator` | `PixCalculator`, `CreditCardCalculator`, `PaymentSimulator` em `src/domain/payment/` |
| **Porta de Provedor** | `interface VehicleDebtProvider` | `#[async_trait] pub trait VehicleDebtProvider: Send + Sync` |
| **Adapters de Protocolo** | `RestVehicleDebtProvider`, `SoapVehicleDebtProvider`, `AiVehicleDebtProvider` | Implementações usando `reqwest::Client`, `serde_json`, `roxmltree` e parser CSV defensivo |
| **Resiliência (Retry/Fallback)** | `ProviderExecutor`, `ProviderResolver`, `VehicleDebtService` | `ProviderExecutor` (com `Sleeper` trait injetável), `ProviderResolver`, `VehicleDebtService` |
| **Métricas em Memória** | `SimpleMetricsRegistry` (Cache + array in-memory) | `SimpleMetricsRegistry` com `std::sync::atomic::AtomicU64` (lock-free, thread-safe) |
| **Cache de Health Check (5s)** | `Cache::get/put('health:integrations:status', 5)` | `tokio::sync::RwLock<Option<CachedHealthEntry>>` com `Instant` TTL de 5s |
| **Logs Estruturados** | Monolog `StructuredJsonFormatter` (`stdout` + `laravel.log`) | `tracing` + camada JSON gravando em `stdout` e `storage/logs/laravel.log` |
| **Interface Web (`GET /`)** | Laravel Livewire (`VehicleDebtLookup` + Blade Tailwind) | `askama` template HTML + Tailwind CSS (via form/HTMX ou fetch integrado) |
| **Diagnóstico CLI** | `php artisan cardok:check-services` | Subcomando `infocar-monolith check-services` (com wrapper `php artisan` opcional no container para 100% de compatibilidade com scripts legados) |

---

## 2. Stack de Crates Recomendada (`monolith/Cargo.toml`)

```toml
[package]
name = "infocar-monolith"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
tokio = { version = "1.43", features = ["full"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["trace", "set-header", "cors"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
roxmltree = "0.20"
csv = "1.3"
regex = "1.11"
uuid = { version = "1.12", features = ["v4"] }
thiserror = "2.0"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
askama = "0.12"

[dev-dependencies]
wiremock = "0.6"
tokio-test = "0.4"
```

---

## 3. Especificação Matemática e Regras de Negócio do Domínio

Para que a comparação entre `cardok` (PHP) e `infocar` (Rust) seja exata ao centavo, o domínio Rust deve reproduzir fielmente as seguintes fórmulas:

### 3.1. Arredondamento Racional Sem Float (`HalfUpRounder`)

Para qualquer fração racional $\frac{\text{numerator}}{\text{denominator}}$ com $\text{denominator} > 0$:
- Se $\text{numerator} \ge 0$:
  $$\text{result} = \left\lfloor \frac{\text{numerator} + \lfloor \text{denominator} / 2 \rfloor}{\text{denominator}} \right\rfloor$$
- Se $\text{numerator} < 0$:
  $$\text{result} = -\left\lfloor \frac{-\text{numerator} + \lfloor \text{denominator} / 2 \rfloor}{\text{denominator}} \right\rfloor$$

Em Rust (divisão inteira `/` já trunca em direção a zero para inteiros positivos):
```rust
pub fn round(numerator: i64, denominator: i64) -> Result<i64, DomainError> {
    if denominator <= 0 {
        return Err(DomainError::InvalidDenominator(denominator));
    }
    if numerator >= 0 {
        Ok((numerator + (denominator / 2)) / denominator)
    } else {
        Ok(-((-numerator + (denominator / 2)) / denominator))
    }
}
```

### 3.2. Cálculo de Dias em Atraso (`days_overdue`)

- Data de referência padrão (`FixedClock`): `2024-05-10` (UTC, `00:00:00`).
- Para cada débito com data de vencimento `due_date` (`YYYY-MM-DD`):
  $$\text{diff} = (\text{ref\_date} - \text{due\_date}).\text{num\_days}()$$
  $$\text{days\_overdue} = \max(0, \text{diff})$$

### 3.3. Políticas de Juros por Tipo de Débito

Seja $C$ o valor original em centavos (`original_amount.cents()`) e $d = \text{days\_overdue}$:

1. **`IPVA` e `LICENCIAMENTO`**:
   - Taxa diária: $0.33\% = \frac{33}{10000}$.
   - Teto: $20\% = \frac{2000}{10000}$.
   - Numerador calculado: $N_{\text{calc}} = C \times 33 \times d$.
   - Numerador do teto: $N_{\text{cap}} = C \times 2000$.
   - Numerador aplicado (teto antes do arredondamento): $N_{\text{app}} = \min(N_{\text{calc}}, N_{\text{cap}})$.
   - Juros em centavos: $J = \text{HalfUpRounder::round}(N_{\text{app}}, 10000)$.
2. **`MULTA`**:
   - Taxa diária: $1.00\% = \frac{1}{100}$, sem teto.
   - Numerador calculado: $N_{\text{calc}} = C \times 1 \times d$.
   - Juros em centavos: $J = \text{HalfUpRounder::round}(N_{\text{calc}}, 100)$.
3. **Valor Atualizado**:
   $$V_{\text{atualizado}} = C + J$$

### 3.4. Simulação de Pagamentos (`PIX` e `Cartão de Crédito`)

1. **Opções Geradas**:
   - Se `debts` for vazio ou `total_updated == 0`, retorna `opcoes: []`.
   - Caso contrário, gera:
     1. `TOTAL`: com `valor_base = total_updated`.
     2. `SOMENTE_<TIPO>`: agrupando a soma de `updated_amount` por categoria **na ordem de primeira aparição** na lista de débitos (ex.: `SOMENTE_IPVA`, `SOMENTE_MULTA`, `SOMENTE_LICENCIAMENTO`).
2. **PIX (5% de desconto)**:
   $$\text{pix\_cents} = \text{HalfUpRounder::round}(C_{\text{base}} \times 95, 100)$$
3. **Cartão de Crédito (1x, 6x, 12x)**:
   - `1x`: $\text{parcela}_1 = C_{\text{base}}$.
   - `6x` e `12x` (Tabela Price com $i = 0.025$ a.m.):
     $$\text{compounded} = (1.0 + 0.025)^n$$
     $$\text{factor} = \frac{0.025 \times \text{compounded}}{\text{compounded} - 1.0}$$
     $$\text{installment\_cents} = \lfloor C_{\text{base}} \times \text{factor} + 0.5 \rfloor$$

---

## 4. Plano de Implementação em 7 Fases

Cada fase possui um guia detalhado na pasta [`docs/rust-phases/`](rust-phases/):

1. **[Fase 1 — Domínio Financeiro Puro](rust-phases/fase-01-dominio-financeiro.md)**:
   Inicialização do Cargo package e implementação de `Money`, `HalfUpRounder`, `DebtType`, `Clock`, `DebtInterestPolicy`, `DebtCalculationService`, `PixCalculator`, `CreditCardCalculator` e `PaymentSimulator` com 100% de testes unitários de domínio.
2. **[Fase 2 — Aplicação, Resiliência e Observabilidade Base](rust-phases/fase-02-aplicacao-resiliencia.md)**:
   Implementação de `PlateMasker`, `SimpleMetricsRegistry` (atômico), `ProviderResolver`, `ProviderExecutor` (retry linear + backoff + fail-fast), `VehicleDebtService` (*First-Success-Wins*) e `GetVehicleDebtsUseCase`.
3. **[Fase 3 — Adaptadores HTTP para os Providers PHP](rust-phases/fase-03-adapters-http-providers.md)**:
   Implementação de `RestVehicleDebtProvider` (JSON), `SoapVehicleDebtProvider` (XML com suporte a `<debts/>`) e `AiVehicleDebtProvider` (CSV defensivo com stripping de Markdown, detecção `;`/`,` e log de categorias ignoradas).
4. **[Fase 4 — Entrega HTTP com Axum, Validação e Middlewares](rust-phases/fase-04-api-http-axum.md)**:
   Endpoints `POST /api/v1/vehicles/debts`, `GET /api/health`, `GET /api/health/integrations` (cache 5s + `X-Cache: HIT|MISS`), `GET /api/metrics`, `RequestIdMiddleware`, `SecurityHeadersMiddleware` e rate limit de 60 req/min.
5. **[Fase 5 — Interface Web (`GET /`) e Diagnóstico CLI](rust-phases/fase-05-web-ui-e-cli.md)**:
   Template HTML Tailwind idêntico ao Livewire original para consulta visual de placas e subcomando CLI `check-services`.
6. **[Fase 6 — Dockerização Multi-Stage e Orquestração Híbrida](rust-phases/fase-06-docker-e-makefile.md)**:
   Configuração do `monolith/Dockerfile` em Rust, cópia/integração dos containers PHP (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`), `docker-compose.yml`, `Makefile` e `scripts/demo-fallback.sh`.
7. **[Fase 7 — Suíte de Testes Completa e Benchmark Comparativo](rust-phases/fase-07-testes-e-comparacao.md)**:
   Testes de contrato e integração, validação dos 6 Gates da Constituição e guia de comparação de performance (latência p50/p95/p99, throughput req/s, RSS de memória e tempo de boot) entre `cardok` (PHP) e `infocar` (Rust).
