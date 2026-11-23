# Constituição do Projeto Infocar (Monólito Rust + Providers PHP)

**Versão:** 2.0.0
**Idioma dos artefatos:** pt-BR
**Status:** Ratificada em 2026-11-23 (Derivada do Cardok v1.0.1 para comparação de stacks PHP vs Rust)

## I. Arquitetura e Isolamento do Domínio — NÃO NEGOCIÁVEL

* `monolith/src/domain` DEVE ser independente de frameworks web (`axum`, `tower`), banco de dados, clientes HTTP (`reqwest`), SOAP/XML, CSV e infraestrutura.
* Regras de negócio, Entities e Value Objects NÃO DEVEM depender de crates de transporte ou de detalhes de protocolo.
* Dependências externas DEVEM ser acessadas por contratos/portas (`traits` definidas no domínio ou aplicação).
* Implementações concretas DEVEM permanecer em `monolith/src/infrastructure`.
* REST (JSON), SOAP (XML) e AI/CSV (Plaintext) DEVEM ser isolados por adapters que implementam a trait `VehicleDebtProvider`.
* Os serviços satélites (`provider-rest`, `provider-soap`, `provider-csv`, `payment-provider`) permanecem em PHP em containers isolados para garantir paridade exata de benchmark e integração com o projeto original (`cardok`).

## II. Precisão Financeira — NÃO NEGOCIÁVEL

* Valores monetários DEVEM utilizar o Value Object `Money` internamente em centavos inteiros (`i64`).
* `f32`/`f64` NÃO DEVEM representar, armazenar ou somar valores monetários.
* Cálculos que exigirem ponto flutuante (`f64`) DEVEM limitar seu uso exclusivamente a coeficientes matemáticos adimensionais (fator exponencial da Tabela Price).
* Valores monetários DEVEM utilizar arredondamento racional estrito `HALF_UP` (`HalfUpRounder`).
* Arredondamentos intermediários DEVEM ser evitados quando puderem alterar o resultado (ex.: o teto de 20% do IPVA e Licenciamento é aplicado sobre o numerador exato antes da divisão e arredondamento).
* A API DEVE retornar valores monetários como strings decimais formatadas com duas casas (`"1500.00"`).

## III. Integrações e Resiliência

* Providers externos DEVEM ser convertidos para o modelo canônico (`ProviderDebtResponse` / `Vec<Debt>`) antes de entrar no domínio.
* Erros transitórios (`ProviderError::Unavailable`) DEVEM possuir retry limitado configurável (`PROVIDER_RETRIES`, padrão `2`).
* Retry DEVE utilizar backoff linear limitado (`tentativa * PROVIDER_BACKOFF_MS`, padrão `100ms`).
* Respostas malformadas (`ProviderError::InvalidResponse`) DEVEM realizar *fail-fast* (sem retry no mesmo provider) e acionar imediatamente o próximo provider.
* Fallback DEVE seguir **First-Success-Wins**: provider primário → retry → próximo provider da lista (`PROVIDER_ORDER`, padrão `rest,soap,csv`).
* Resposta válida com zero débitos NÃO DEVE disparar fallback.
* Providers NÃO DEVEM ser consultados concorrentemente no fluxo padrão.

## IV. Simulação de Pagamentos

O sistema DEVE somente simular pagamentos no escopo atual.

* PIX: desconto de 5% sobre o valor base atualizado, arredondado via `HALF_UP`.
* Cartão: somente `1x` (sem juros), `6x` e `12x`.
* Parcelamento (6x e 12x): Tabela Price, taxa composta de `2,5% a.m.` (`0.025`), arredondando cada parcela via `HALF_UP`.
* Opções geradas: `TOTAL` e `SOMENTE_<TIPO>` (agrupadas por categoria na ordem de primeira aparição dos débitos).
* Se não houver débitos ou o total atualizado for zero, `opcoes` DEVE ser uma lista vazia (`[]`).

O sistema NÃO DEVE executar cobranças reais, gerar PIX real ou conectar-se a adquirentes reais.

## V. Determinismo Temporal

* Regras dependentes de tempo DEVEM utilizar a trait `Clock` injetável.
* O domínio NÃO DEVE utilizar diretamente `Utc::now()` ou `Local::now()`.
* A data de referência padrão do sistema é `2024-05-10T00:00:00Z` (`FixedClock`).
* Comparações de data DEVEM utilizar UTC truncado no início do dia (`00:00:00`).

## VI. Observabilidade e Privacidade

* Requisições DEVEM possuir e propagar `X-Request-ID` (validando formato `^[a-zA-Z0-9\-_]{1,64}$` ou gerando UUID v4).
* Logs DEVEM ser estruturados em JSON contendo `request_id`, `event`, duração, tentativa, provider, fallback e resultado.
* Placas DEVEM ser mascaradas em todos os logs via `PlateMasker` (`ABC1234` → `ABC****`).
* Dados pessoais DEVEM ser minimizados conforme princípios da LGPD.

## VII. Testes

* Regras de negócio, cálculos financeiros, integrações, parsers (JSON, XML, CSV), retry, fallback e contratos de API DEVEM possuir testes automatizados em Rust (`cargo test`).
* A suíte de testes DEVE passar integralmente com zero warnings de compilação antes de qualquer entrega.
* Os resultados matemáticos DEVEM ser 100% idênticos aos testes de referência do monólito PHP (`cardok`).

## VIII. Artefatos de Engenharia

* Constitution, specs, plans, tasks, ADRs, README e demais documentos DEVEM ser escritos em **pt-BR**.
* Identificadores técnicos, nomes de structs, traits, funções, APIs, comandos, crates e protocolos DEVEM permanecer em sua forma original em inglês.

## IX. Desenvolvimento Assistido por IA

* Features relevantes DEVEM seguir o ciclo Spec Kit:

```text
specify → plan → tasks → implement → validate
```

* Código gerado por IA DEVE obedecer às mesmas regras arquiteturais e de qualidade do código escrito manualmente.
* Alterações DEVEM ser validadas por testes e pelos gates definidos nesta Constituição.

## X. Gates de Qualidade

Antes de concluir uma alteração, verificar:

1. **Arquitetura:** `src/domain` isolado e sem dependências de transporte/infraestrutura.
2. **Financeiro:** precisão em centavos inteiros (`i64`) e `HALF_UP` preservados.
3. **Testes:** `cargo test` passando 100%.
4. **Integração:** retry linear, fail-fast e fallback *First-Success-Wins* preservados.
5. **Docker:** `docker compose config` válido e containers saudáveis (`monolith` em Rust + `provider-*` em PHP).
6. **Contrato:** requests, responses, headers (`X-Request-ID`, `X-Cache`, Security Headers) e códigos HTTP (200, 400, 422, 503) 100% compatíveis com a especificação do Cardok.
