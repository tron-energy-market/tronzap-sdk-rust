# Aluguel de Energia Tron via API
## SDK Rust por TronZap.com

[English](README.md) | [Español](README.es.md) | **[Português](README.pt-br.md)** | [Русский](README.ru.md)

[![crates.io](https://img.shields.io/crates/v/tronzap-sdk.svg)](https://crates.io/crates/tronzap-sdk)
[![docs.rs](https://img.shields.io/docsrs/tronzap-sdk)](https://docs.rs/tronzap-sdk)
[![CI](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

SDK oficial em Rust para a API do TronZap.
Este SDK permite integrar facilmente os serviços TronZap para aluguel de energia TRON.

TronZap.com permite [comprar energia TRON](https://tronzap.com/), reduzindo significativamente as taxas nas transferências de USDT (TRC20).

👉 [Registre-se para obter uma chave API](https://tronzap.com) para começar a usar a API TronZap e integrá-la através do SDK.

- Site: https://tronzap.com/
- Referência da API: https://docs.tronzap.com/
- crates.io: https://crates.io/crates/tronzap-sdk
- Documentação: https://docs.rs/tronzap-sdk
- Código-fonte: https://github.com/tron-energy-market/tronzap-sdk-rust

## Instalação

```toml
[dependencies]
tronzap-sdk = "1.2"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Requisitos

- Rust 1.85 ou mais recente
- Um runtime [Tokio](https://tokio.rs/): as requisições passam pelo [`reqwest`](https://crates.io/crates/reqwest). O SDK não inicia um runtime por conta própria.

## Início rápido

```rust,no_run
use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
use tronzap_sdk::TronzapClient;

#[tokio::main]
async fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("seu_api_token")
        .api_secret("seu_api_secret")
        .build()?;

    let balance = client.get_balance().await?;
    println!("balance: {} (deposit to {})", balance.balance, balance.address);

    // Estima quanta energia uma transferência de USDT precisa e compra exatamente essa quantidade.
    let estimate = client
        .estimate_energy(&EstimateEnergyRequest::new("TSenderAddress", "TRecipientAddress"))
        .await?;

    let request = EnergyTransactionRequest::new("TRecipientAddress", estimate.amount)
        .duration(1)
        .external_id("order-42")
        .activate_address(true);
    let tx = client.create_energy_transaction(&request).await?;
    println!("transaction {} costs {} and is {}", tx.id, tx.amount, tx.status);
    Ok(())
}
```

Um passo a passo executável de todas as operações está em
[`examples/basic_usage.rs`](examples/basic_usage.rs):

```bash
export TRONZAP_API_TOKEN=seu_api_token
export TRONZAP_API_SECRET=seu_api_secret
export TRONZAP_BASE_URL=api.tronzap.com   # opcional
cargo run --example basic_usage
```

Por padrão ele apenas lê e não gasta nada. Com `TRONZAP_ALLOW_PURCHASES=1` ele
também executa os endpoints que criam transações e verificações AML, que debitam
o saldo da conta. As demais variáveis opcionais estão descritas no comentário no
início do arquivo.

## Configuração

O builder recebe as duas credenciais do seu painel: o token da API é enviado como
bearer token e o segredo da API assina o corpo de cada requisição. Todo o resto é
opcional:

```rust
use std::time::Duration;
use tronzap_sdk::TronzapClient;

fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("seu_api_token")
        .api_secret("seu_api_secret")
        .base_url("api.tronzap.com")        // padrão: tronzap_sdk::DEFAULT_BASE_URL
        .timeout(Duration::from_secs(10))   // por requisição; padrão: 30 segundos
        .user_agent("my-app/1.0")
        .build()?;
    Ok(())
}
```

`base_url` aceita um domínio ou uma URL completa: sem esquema, usa-se `https`, e a
barra final é removida, então `"api.tronzap.com"`, `"api.tronzap.com/"` e
`"https://api.tronzap.com"` são equivalentes. Informe um esquema explícito para
evitar isso, por exemplo `"http://localhost:8080"` com um mock local.
`TronzapClient::new(token, secret)` é um atalho com os valores padrão.

Um `TronzapClient` é barato de clonar, não guarda estado mutável e pode ser usado
por várias tarefas ao mesmo tempo, então crie um por conjunto de credenciais e
compartilhe-o. A saída `Debug` dele nunca mostra o token nem o segredo.

### Seu próprio cliente reqwest

Para usar um proxy, certificados próprios ou ajustes do pool de conexões, passe um
`reqwest::Client`. Use o `reqwest` reexportado pelo SDK para que as versões sempre
coincidam:

```rust
use tronzap_sdk::{reqwest, TronzapClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https("http://proxy.internal:3128")?)
        .build()?;

    let client = TronzapClient::builder()
        .api_token("seu_api_token")
        .api_secret("seu_api_secret")
        .http_client(http)
        .build()?;
    Ok(())
}
```

O cliente é usado como está: os cabeçalhos são definidos a cada requisição. O
timeout dele continua valendo, a menos que você também chame `.timeout(...)`, que o
substitui.

### Features

| Feature | Padrão | Descrição |
|---|---|---|
| `rustls` | sim | TLS via rustls, com o verificador de certificados da plataforma |
| `native-tls` | não | TLS via biblioteca da plataforma (OpenSSL, Secure Transport, SChannel). Com as duas features de TLS ativas, esta é usada |
| `system-proxy` | não | Respeitar as configurações de proxy do sistema operacional |

```toml
tronzap-sdk = { version = "1.2", default-features = false, features = ["native-tls"] }
```

## Métodos disponíveis

| Método | Endpoint | Descrição |
|---|---|---|
| `get_services()` | `/v1/services` | Serviços disponíveis e preços |
| `get_balance()` | `/v1/balance` | Saldo atual da conta |
| `get_address_info(address)` | `/v1/address-info` | Recursos do endereço (energy, bandwidth) e saldos (TRX, USDT) |
| `estimate_energy(&request)` | `/v1/estimate-energy` | Energia de que uma transferência precisa e seu custo |
| `calculate(&request)` | `/v1/calculate` | Preço de uma compra sem criar uma transação |
| `create_energy_transaction(&request)` | `/v1/transaction/new` | Comprar energia |
| `create_bandwidth_transaction(&request)` | `/v1/transaction/new` | Comprar bandwidth |
| `create_resource_bundle_transaction(&request)` | `/v1/transaction/new` | Comprar energia e bandwidth em uma única transação |
| `create_address_activation_transaction(&request)` | `/v1/transaction/new` | Ativar um endereço TRON |
| `check_transaction(&request)` | `/v1/transaction/check` | Status de uma transação, por id ou external id |
| `get_direct_recharge_info()` | `/v1/direct-recharge-info` | Endereço e tarifas de recarga direta |
| `get_aml_services()` | `/v1/aml-checks` | Serviços AML e preços |
| `create_aml_check(&request)` | `/v1/aml-checks/new` | Iniciar uma verificação AML |
| `check_aml_status(id)` | `/v1/aml-checks/check` | Status e resultado de uma verificação AML |
| `get_aml_history(&request)` | `/v1/aml-checks/history` | Histórico paginado de verificações AML |
| `get_subscriptions()` | `/v1/subscriptions` | Planos de assinatura e preços |
| `start_subscription(&request)` | `/v1/subscription/start` | Assinar um plano para um endereço |
| `check_subscription(&request)` | `/v1/subscription/check` | Status de uma assinatura, por id ou id externo |
| `stop_subscription(&request)` | `/v1/subscription/stop` | Interromper uma assinatura |
| `get_subscription_history(&request)` | `/v1/subscriptions/history` | Histórico paginado de assinaturas |
| `bulk_calculate(&request)` | `/v1/bulk/calculate` | Validar e calcular as linhas de um pedido de energia em massa |
| `bulk_create(&request)` | `/v1/bulk/create` | Iniciar um pedido em massa a partir de um cálculo |
| `bulk_check(&request)` | `/v1/bulk/check` | Status e progresso de um pedido em massa |
| `get_bulk_history(&request)` | `/v1/bulk/history` | Histórico paginado de pedidos em massa |
| `bulk_report(&request)` | `/v1/bulk/report` | Relatório por endereço de um pedido em massa, em CSV |
| `verify_webhook_signature(body, signature)` | — | Verificar o `X-Signature` do webhook de um pedido em massa |
| `request(endpoint, &params)` | qualquer | Chamada assinada a um endpoint que o SDK ainda não cobre |

Todo método é `async` e retorna `Result<T, TronzapError>`. Descartar o future
cancela a requisição.

Os parâmetros ficam nos tipos de requisição de `tronzap_sdk::requests`. Valores
obrigatórios são argumentos de `new` (ou de um construtor nomeado, como
`CheckTransactionRequest::by_external_id`), e os opcionais são setters
encadeáveis. A requisição é validada antes do envio, então uma inválida retorna
`TronzapError::Validation` e nunca chega à API. Os valores padrão coincidem com a
API: `duration` é 1 hora, e os históricos AML e de
assinaturas começam na página 1 com 10 itens. A exceção é
`StartSubscriptionRequest`, em que `duration_days` ou `transactions_limit` igual
a zero significa sem limite.

Os resultados são structs de `tronzap_sdk::responses`. Listas nunca faltam, e
valores que a API pode omitir são `Option`.

### Comprar recursos

```rust,no_run
use tronzap_sdk::requests::{
    AddressActivationRequest, BandwidthTransactionRequest, EnergyTransactionRequest,
    ResourceBundleTransactionRequest,
};
use tronzap_sdk::TronzapClient;

async fn buy(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    // Energia, opcionalmente ativando o endereço na mesma chamada.
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
        .duration(1) // horas; veja get_services() para as durações disponíveis
        .external_id("order-42")
        .activate_address(true);
    client.create_energy_transaction(&request).await?;

    // Bandwidth.
    let request = BandwidthTransactionRequest::new("TRecipientAddress", 345).external_id("bandwidth-1");
    client.create_bandwidth_transaction(&request).await?;

    // Energia e bandwidth juntos em uma única transação.
    let request = ResourceBundleTransactionRequest::new("TRecipientAddress", 65000, 345).external_id("bundle-1");
    client.create_resource_bundle_transaction(&request).await?;

    // Apenas a ativação.
    let request = AddressActivationRequest::new("TRecipientAddress").external_id("activation-1");
    client.create_address_activation_transaction(&request).await?;
    Ok(())
}
```

Os preços da energia e do bandwidth são ambos por 1000 unidades: em
`get_services()`, 65000 de energia com `EnergyRate::price` igual a 0.03 custam
0.03 × 65000 / 1000 = 1.95, e 345 de bandwidth com `BandwidthRate::price` igual a 1
custam 0.345.

Atualmente a API retorna um resource bundle com `service` igual a
`Service::Energy`, e não `Service::ResourceBundle`. Leia `params.amounts` para
saber quais recursos uma transação contém.

### Acompanhar uma transação

Uma transação passa por `New` → `Pending` → `Success` ou `Failed`:

```rust,no_run
use std::time::Duration;
use tronzap_sdk::models::TransactionStatus;
use tronzap_sdk::requests::CheckTransactionRequest;
use tronzap_sdk::TronzapClient;

async fn wait(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let request = CheckTransactionRequest::by_external_id("order-42");
    let tx = loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let tx = client.check_transaction(&request).await?;
        if !matches!(tx.status, TransactionStatus::New | TransactionStatus::Pending) {
            break tx;
        }
    };
    println!("finished as {}, hash {}", tx.status, tx.hash.as_deref().unwrap_or("none"));
    Ok(())
}
```

### Verificação AML

```rust,no_run
use tronzap_sdk::models::AmlStatus;
use tronzap_sdk::requests::AmlCheckRequest;
use tronzap_sdk::TronzapClient;

async fn screen(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let check = client.create_aml_check(&AmlCheckRequest::for_address("TRX", "TAddressToScreen")).await?;
    // ou AmlCheckRequest::for_hash("BTC", "bc1RecipientAddress", "TX_HASH", AmlDirection::Withdrawal)

    let result = client.check_aml_status(&check.id).await?;
    if result.status == AmlStatus::Completed {
        println!("{:?} {:?} {} factor(s)", result.risk_level, result.risk_score, result.risk_factors.len());
    }
    Ok(())
}
```

Em uma verificação por hash, `address` é o endereço do destinatário da
transação, onde os fundos foram recebidos, e a direção indica de que lado você
está: `Deposit` se os fundos chegaram ao seu endereço (`address` é o seu
endereço), `Withdrawal` se foi você quem enviou (`address` é o endereço do
destinatário externo). O risco é calculado para a contraparte: o remetente em um
deposit, o destinatário em um withdrawal.

`risk_score` é `None` até a verificação terminar. Uma verificação concluída pode
ter score 0, o que não é o mesmo que ainda não ter score.

### Assinaturas

Uma assinatura mantém um endereço abastecido de energia para cada transação até
ser interrompida ou esgotar seus dias ou transações. Escolha um plano de
`get_subscriptions` e passe o seu `subscription_id`, como `"unlimited_energy"`,
não o `id` numérico:

```rust,no_run
use tronzap_sdk::models::SubscriptionStatus;
use tronzap_sdk::requests::{StartSubscriptionRequest, SubscriptionHistoryRequest, SubscriptionRequest};
use tronzap_sdk::TronzapClient;

async fn subscribe(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    for plan in client.get_subscriptions().await? {
        println!("{} {} {}", plan.subscription_id, plan.initial_price, plan.price);
    }

    let request = StartSubscriptionRequest::new("unlimited_energy", "TRecipientAddress")
        .duration_days(30) // 0 para sem limite de tempo
        .transactions_limit(0) // 0 para sem limite
        .external_id("subscription-42");
    let started = client.start_subscription(&request).await?;
    println!("{} {}", started.id, started.status);

    let sub = client.check_subscription(&SubscriptionRequest::by_external_id("subscription-42")).await?;

    let stopped = client.stop_subscription(&SubscriptionRequest::by_id(&sub.id)).await?;

    let history = client
        .get_subscription_history(&SubscriptionHistoryRequest::new().status(SubscriptionStatus::Active))
        .await?;
    println!("{} {}", stopped.status, history.total);
    Ok(())
}
```

Iniciar, consultar e interromper retornam a assinatura com seus `params`; o
histórico retorna em vez disso os contadores de uso `transactions_used`,
`energy_used` e `total_price`, com `params` igual a `None`. Uma assinatura com
limite de transações não pode ser interrompida
(`ErrorCode::CannotStopSubscription`).

### Pedidos em massa

Compre energia para até 10 000 endereços em um único pedido. `bulk_calculate`
valida e calcula as linhas e retorna um cálculo válido por 10 minutos; nada é
cobrado. Linhas inválidas não fazem a requisição falhar: elas voltam em `rejected`
com o número da linha. `bulk_create` inicia o pedido; cada endereço é cobrado
quando é processado, e endereços não ativados são pulados sem cobrança.

Se o preço mudou desde o cálculo, `bulk_create` não inicia o pedido e falha com
`ErrorCode::BulkPriceChanged`. `ApiError::bulk_calculation()` é o novo cálculo:
confirme o novo total e inicie-o pelo `id`.

```rust,no_run
use tronzap_sdk::requests::{
    BulkCalculateRequest, BulkCreateRequest, BulkItem, BulkOrderRequest, BulkReportRequest,
};
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn bulk(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let request = BulkCalculateRequest::new([
        BulkItem::new("TRecipientAddress1", 65000),
        BulkItem::new("TRecipientAddress2", 131000),
    ]);
    let calculation = client.bulk_calculate(&request).await?;
    for row in &calculation.rejected {
        println!("row {}: {}", row.row, row.reason);
    }

    let request = BulkCreateRequest::new(&calculation.id)
        .external_id("order-42")
        .webhook_url("https://example.com/tronzap/bulk");
    let order = match client.bulk_create(&request).await {
        Err(TronzapError::Api(e)) if e.error_code() == Some(ErrorCode::BulkPriceChanged) => {
            match e.bulk_calculation() {
                // confirme o novo total e inicie-o
                Some(new_calculation) => {
                    let request = BulkCreateRequest::new(&new_calculation.id).external_id("order-42");
                    client.bulk_create(&request).await?
                }
                None => return Err(TronzapError::Api(e)),
            }
        }
        result => result?,
    };

    // consulte até Completed, Partial ou Failed
    let order = client.bulk_check(&BulkOrderRequest::by_external_id("order-42")).await?;
    // row,address,amount,status,reason,charged,transaction_id
    let csv = client.bulk_report(&BulkReportRequest::by_id(&order.id)).await?;
    println!("{} {}", order.status, csv.lines().count());
    Ok(())
}
```

Um pedido passa pelos valores de `BulkStatus`; `Completed`, `Partial` e `Failed`
são finais. `BulkReportRequest::status` filtra o relatório por `BulkRowStatus`.

Com `webhook_url`, a API envia um `POST` quando o pedido chega a um status final.
Verifique o cabeçalho `X-Signature` com o corpo bruto antes de confiar nele, no
framework web que você usa:

```rust,no_run
use tronzap_sdk::TronzapClient;

fn bulk_webhook(client: &TronzapClient, body: &[u8], x_signature: Option<&str>) -> u16 {
    if !client.verify_webhook_signature(body, x_signature.unwrap_or_default()) {
        return 401;
    }
    // {"id":"01kb2m5x9qj4v8n3c7d6e5f4g3","external_id":"order-42","status":"partial"}
    let order: tronzap_sdk::serde_json::Value = match tronzap_sdk::serde_json::from_slice(body) {
        Ok(order) => order,
        Err(_) => return 400,
    };
    println!("{} {}", order["id"], order["status"]);
    200
}
```

## Tratamento de erros

Toda falha é um `TronzapError`. Faça match nas variantes para tratar um tipo
específico de falha:

```text
TronzapError
├── Transport(TransportError)        — nenhuma resposta chegou
│     kind(): Timeout | Connect | Tls | Other
├── Api(ApiError)                    — a API respondeu com um code diferente de zero ou um status não 2xx
├── Authentication(ApiError)         — código de erro 1, ou HTTP 401 / 403
├── Validation(ValidationError)      — argumentos inválidos; nada foi enviado
└── Serialization(SerializationError) — resposta 2xx que o SDK não conseguiu ler
```

`ApiError` traz o status HTTP (`status()`), o código de erro da API (`code()` e
`error_code()`), a chave do erro, a mensagem, o ID da requisição, o corpo bruto da
resposta (`body()`), o `result` enviado com o erro (`result()`) e quanto esperar
antes de tentar de novo (`retry_after()`). `SerializationError` também traz o
status e o corpo bruto.

```rust,no_run
use tronzap_sdk::requests::EnergyTransactionRequest;
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn buy(client: &TronzapClient) {
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000);
    match client.create_energy_transaction(&request).await {
        Ok(tx) => println!("bought, transaction {}", tx.id),
        // Falha no nível da aplicação: o código diz exatamente o que deu errado.
        Err(TronzapError::Api(e)) => match e.error_code() {
            // A chave pode detalhar, por exemplo "invalid_tron_address.from_address"
            Some(ErrorCode::InvalidTronAddress) => eprintln!("bad address: {:?}", e.key()),
            Some(ErrorCode::InsufficientFunds) => eprintln!("top up the account"),
            Some(ErrorCode::AddressNotActivated) => eprintln!("activate the address first"),
            // Aguarde e tente de novo, após e.retry_after() se a API o enviou.
            _ if e.is_rate_limited() => eprintln!("rate limited, retry after {:?}", e.retry_after()),
            // Transitório; pode tentar de novo.
            _ if e.is_server_error() => eprintln!("server error, retry later"),
            _ => eprintln!("api error {:?}: {} (request {:?})", e.code(), e.message(), e.request_id()),
        },
        // Token ou assinatura incorretos.
        Err(TronzapError::Authentication(e)) => eprintln!("check the credentials: {e}"),
        // Transitório; pode tentar de novo.
        Err(TronzapError::Transport(e)) if e.is_timeout() => eprintln!("timed out"),
        Err(e) => eprintln!("{e}"),
    }
}
```

`request_id()` é o identificador que a API atribui a cada requisição. Informe-o ao
contatar o suporte.

Um erro da API tem prioridade sobre o status HTTP: a API informa algumas falhas
com status 2xx e outras com 4xx ou 5xx, então um payload legível com código
diferente de zero é sempre informado com esse código, nunca como um simples erro
HTTP.

`ApiError::result()` é o `result` que a API enviou com o erro, como
`serde_json::Value`. O SDK lê os dados de `BulkPriceChanged` e `BulkNoValidItems`
em `ApiError::bulk_calculation()` e `ApiError::bulk_rejected()`. Um limite de
requisições é um `ApiError` com `ErrorCode::RateLimited` da API, ou um HTTP 429 sem
código, p. ex. de um proxy; `is_rate_limited()` é `true` nos dois casos.
`retry_after()` vem do cabeçalho `Retry-After` (segundos ou uma data HTTP) ou do
campo `retry_after`, e é `None` quando o servidor não informou quanto esperar:

```rust,no_run
use std::time::Duration;

use tronzap_sdk::requests::BulkHistoryRequest;
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn history(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    match client.get_bulk_history(&BulkHistoryRequest::new()).await {
        Err(TronzapError::Api(e)) if e.error_code() == Some(ErrorCode::RateLimited) => {
            tokio::time::sleep(e.retry_after().unwrap_or(Duration::from_secs(60))).await;
        }
        result => println!("{} bulk order(s)", result?.total),
    }
    Ok(())
}
```

### Códigos de erro da API

| Código | Constante | Descrição |
|------|----------|-------------|
| 1 | `AuthError` | Erro de autenticação: token da API ou assinatura inválidos |
| 2 | `InvalidServiceOrParams` | Serviço ou parâmetros inválidos |
| 5 | `WalletNotFound` | Carteira interna não encontrada. Contate o suporte. |
| 6 | `InsufficientFunds` | Saldo insuficiente |
| 10 | `InvalidTronAddress` | Endereço TRON inválido, ou o endereço já tem uma assinatura ativa |
| 11 | `InvalidEnergyAmount` | Quantidade de energia inválida |
| 12 | `InvalidDuration` | Duração inválida |
| 20 | `TransactionNotFound` | Transação/assinatura não encontrada |
| 21 | `CannotStopSubscription` | Não é possível interromper a assinatura, p. ex. ela tem limite de transações |
| 24 | `AddressNotActivated` | Endereço não ativado |
| 25 | `AddressAlreadyActivated` | Endereço já ativado |
| 30 | `AmlCheckNotFound` | Verificação AML não encontrada |
| 35 | `ServiceNotAvailable` | Serviço indisponível |
| 50 | `InvalidBandwidthAmount` | Quantidade de bandwidth inválida |
| 90 | `BulkNotFound` | Pedido em massa não encontrado |
| 91 | `BulkExpired` | O cálculo expirou, calcule novamente |
| 92 | `BulkPriceChanged` | O preço mudou; o novo cálculo está em `bulk_calculation()` |
| 93 | `BulkTooManyItems` | Mais de 10 000 linhas |
| 94 | `BulkNoValidItems` | Nenhuma linha válida; elas estão em `bulk_rejected()` |
| 95 | `BulkResourceNotSupported` | Só energia pode ser pedida em massa |
| 96 | `PayloadTooLarge` | Corpo da requisição grande demais (HTTP 413) |
| 429 | `RateLimited` | Requisições demais; aguarde `retry_after()` se informado |
| 500 | `InternalServerError` | Erro interno do servidor: contate o suporte |

As constantes são variantes do enum `ErrorCode`. Para um código que esta versão do
SDK não conhece, `error_code()` retorna `None`, e o número continua disponível em
`code()`.

## Campos decimais e de data

Valores e preços são [`rust_decimal::Decimal`](https://docs.rs/rust_decimal),
reexportado como `tronzap_sdk::models::Decimal`: aritmética decimal exata e
`1.50 == 1.5`. A API codifica dinheiro como número JSON em algumas respostas e
como string JSON em outras; as duas formas são lidas da mesma maneira. Por
exemplo, o custo de 65000 de energia é
`rate.price * Decimal::from(65000) / Decimal::from(1000)`.

Datas são `Timestamp`: `as_str()` é o texto exatamente como a API enviou, e
`unix_timestamp()` e `to_system_time()` o interpretam. Os vários formatos usados
pela API são aceitos, e horários sem fuso são lidos como UTC. Para uma data não
reconhecida os parsers retornam `None` em vez de fazer toda a resposta falhar.

Valores que a API venha a adicionar no futuro, como um novo status de transação,
são informados como a variante `Unknown(String)` do enum correspondente em vez de
falhar.

## Testes

```bash
cargo test --all-features
```

Executa os testes unitários e os de integração contra um servidor HTTP local: o
corpo exato e a assinatura de cada endpoint, erros de API e HTTP, JSON malformado,
timeouts, falhas de rede e de TLS, e uso concorrente.

## Licença

Licença MIT (MIT). Veja o [arquivo de licença](LICENSE) para mais informações.

## Suporte

Para suporte, entre em contato com [support@tronzap.com](mailto:support@tronzap.com).
