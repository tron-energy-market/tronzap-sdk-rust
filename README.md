# Tron Energy Rental via API
## Rust SDK by TronZap.com

**[English](README.md)** | [Español](README.es.md) | [Português](README.pt-br.md) | [Русский](README.ru.md)

[![crates.io](https://img.shields.io/crates/v/tronzap-sdk.svg)](https://crates.io/crates/tronzap-sdk)
[![docs.rs](https://img.shields.io/docsrs/tronzap-sdk)](https://docs.rs/tronzap-sdk)
[![CI](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Official Rust SDK for the TronZap API.
This SDK allows you to easily integrate with TronZap services for TRON energy rental.

TronZap.com allows you to [buy TRON energy](https://tronzap.com/), making USDT (TRC20) transfers cheaper by significantly reducing transaction fees.

👉 [Register for an API key](https://tronzap.com) to start using TronZap API and integrate it via the SDK.

- Website: https://tronzap.com/
- API reference: https://docs.tronzap.com/
- crates.io: https://crates.io/crates/tronzap-sdk
- Documentation: https://docs.rs/tronzap-sdk
- Source: https://github.com/tron-energy-market/tronzap-sdk-rust

## Installation

```toml
[dependencies]
tronzap-sdk = "1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Requirements

- Rust 1.85 or newer
- A [Tokio](https://tokio.rs/) runtime: requests go through [`reqwest`](https://crates.io/crates/reqwest). The SDK does not start a runtime itself.

## Quick start

```rust,no_run
use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
use tronzap_sdk::TronzapClient;

#[tokio::main]
async fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("your_api_token")
        .api_secret("your_api_secret")
        .build()?;

    let balance = client.get_balance().await?;
    println!("balance: {} (deposit to {})", balance.balance, balance.address);

    // Estimate how much energy a USDT transfer needs, then buy exactly that much.
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

A runnable walkthrough of every operation lives in
[`examples/basic_usage.rs`](examples/basic_usage.rs):

```bash
export TRONZAP_API_TOKEN=your_api_token
export TRONZAP_API_SECRET=your_api_secret
export TRONZAP_BASE_URL=api.tronzap.com   # optional
cargo run --example basic_usage
```

By default it only reads and spends nothing. Setting `TRONZAP_ALLOW_PURCHASES=1`
also exercises the endpoints that create transactions and AML checks, which debit
the account balance. See the comment at the top of the file for the other optional
variables.

## Configuration

The builder takes the two credentials from your dashboard: the API token is sent
as a bearer token, and the API secret signs every request body. Everything else is
optional:

```rust
use std::time::Duration;
use tronzap_sdk::TronzapClient;

fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("your_api_token")
        .api_secret("your_api_secret")
        .base_url("api.tronzap.com")        // defaults to tronzap_sdk::DEFAULT_BASE_URL
        .timeout(Duration::from_secs(10))   // per request; defaults to 30 seconds
        .user_agent("my-app/1.0")
        .build()?;
    Ok(())
}
```

`base_url` takes either a bare domain or a full URL: a missing scheme becomes
`https` and a trailing slash is trimmed, so `"api.tronzap.com"`,
`"api.tronzap.com/"` and `"https://api.tronzap.com"` are equivalent. Pass an
explicit scheme to opt out, for example `"http://localhost:8080"` against a local
mock. `TronzapClient::new(token, secret)` is a shortcut for the defaults.

A `TronzapClient` is cheap to clone, holds no mutable state and is safe to use
from many tasks at once, so create one per set of credentials and share it. Its
`Debug` output never shows the token or the secret.

### Your own reqwest client

To use a proxy, custom certificates or connection pool settings, pass a
`reqwest::Client`. Take `reqwest` from the SDK's re-export so the versions always
match:

```rust
use tronzap_sdk::{reqwest, TronzapClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https("http://proxy.internal:3128")?)
        .build()?;

    let client = TronzapClient::builder()
        .api_token("your_api_token")
        .api_secret("your_api_secret")
        .http_client(http)
        .build()?;
    Ok(())
}
```

The client is used as given: headers are set per request. Its own timeout stays in
effect unless you also call `.timeout(...)`, which replaces it.

### Features

| Feature | Default | Description |
|---|---|---|
| `rustls` | yes | TLS through rustls, with the platform's certificate verifier |
| `native-tls` | no | TLS through the platform's library (OpenSSL, Secure Transport, SChannel). With both TLS features on, this one is used |
| `system-proxy` | no | Honour the operating system's proxy settings |

```toml
tronzap-sdk = { version = "1.0", default-features = false, features = ["native-tls"] }
```

## Available methods

| Method | Endpoint | Description |
|---|---|---|
| `get_services()` | `/v1/services` | Available services and prices |
| `get_balance()` | `/v1/balance` | Current account balance |
| `get_address_info(address)` | `/v1/address-info` | Address resources (energy, bandwidth) and balances (TRX, USDT) |
| `estimate_energy(&request)` | `/v1/estimate-energy` | Energy a transfer needs, and its cost |
| `calculate(&request)` | `/v1/calculate` | Price a purchase without creating a transaction |
| `create_energy_transaction(&request)` | `/v1/transaction/new` | Buy energy |
| `create_bandwidth_transaction(&request)` | `/v1/transaction/new` | Buy bandwidth |
| `create_resource_bundle_transaction(&request)` | `/v1/transaction/new` | Buy energy and bandwidth in one transaction |
| `create_address_activation_transaction(&request)` | `/v1/transaction/new` | Activate a TRON address |
| `check_transaction(&request)` | `/v1/transaction/check` | Status of a transaction, by id or external id |
| `get_direct_recharge_info()` | `/v1/direct-recharge-info` | Direct recharge address and rates |
| `get_aml_services()` | `/v1/aml-checks` | AML services and pricing |
| `create_aml_check(&request)` | `/v1/aml-checks/new` | Start an AML screening |
| `check_aml_status(id)` | `/v1/aml-checks/check` | Status and result of an AML check |
| `get_aml_history(&request)` | `/v1/aml-checks/history` | Paginated AML check history |
| `request(endpoint, &params)` | any | Signed call to an endpoint the SDK does not wrap yet |

Every method is `async` and returns `Result<T, TronzapError>`. Dropping the future
cancels the request.

Parameters live in request types in `tronzap_sdk::requests`. Required values are
arguments of `new` (or of a named constructor such as
`CheckTransactionRequest::by_external_id`), optional ones are chainable setters.
A request is validated before it is sent, so an invalid one returns
`TronzapError::Validation` and never reaches the API. Defaults match the API:
`duration` is 1 hour, and AML history starts at page 1 with 10 items.

Results are structs in `tronzap_sdk::responses`. Lists are never missing, and
values the API may omit are `Option`s.

### Buying resources

```rust,no_run
use tronzap_sdk::requests::{
    AddressActivationRequest, BandwidthTransactionRequest, EnergyTransactionRequest,
    ResourceBundleTransactionRequest,
};
use tronzap_sdk::TronzapClient;

async fn buy(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    // Energy, optionally activating the address in the same call.
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
        .duration(1) // hours; see get_services() for the durations on sale
        .external_id("order-42")
        .activate_address(true);
    client.create_energy_transaction(&request).await?;

    // Bandwidth.
    let request = BandwidthTransactionRequest::new("TRecipientAddress", 345).external_id("bandwidth-1");
    client.create_bandwidth_transaction(&request).await?;

    // Energy and bandwidth together in one transaction.
    let request = ResourceBundleTransactionRequest::new("TRecipientAddress", 65000, 345).external_id("bundle-1");
    client.create_resource_bundle_transaction(&request).await?;

    // Activation on its own.
    let request = AddressActivationRequest::new("TRecipientAddress").external_id("activation-1");
    client.create_address_activation_transaction(&request).await?;
    Ok(())
}
```

Energy and bandwidth prices are both per 1000 units: in `get_services()`, 65000
energy at an `EnergyRate::price` of 0.03 costs 0.03 × 65000 / 1000 = 1.95, and 345
bandwidth at a `BandwidthRate::price` of 1 costs 0.345.

The API currently reports a resource bundle with `service` equal to
`Service::Energy`, not `Service::ResourceBundle`. Read `params.amounts` to see
which resources a transaction contains.

### Following a transaction

A transaction moves through `New` → `Pending` → `Success` or `Failed`:

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

### AML screening

```rust,no_run
use tronzap_sdk::models::AmlStatus;
use tronzap_sdk::requests::AmlCheckRequest;
use tronzap_sdk::TronzapClient;

async fn screen(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let check = client.create_aml_check(&AmlCheckRequest::for_address("TRX", "TAddressToScreen")).await?;
    // or AmlCheckRequest::for_hash("BTC", "bc1RecipientAddress", "TX_HASH", AmlDirection::Withdrawal)

    let result = client.check_aml_status(&check.id).await?;
    if result.status == AmlStatus::Completed {
        println!("{:?} {:?} {} factor(s)", result.risk_level, result.risk_score, result.risk_factors.len());
    }
    Ok(())
}
```

For a hash check, `address` is the recipient address of the transaction, where
the funds were received, and the direction says which side you are on: `Deposit`
if the funds were sent to your address (`address` is your address), `Withdrawal`
if you sent them (`address` is the external recipient's address). The risk is
scored for the counterparty: the sender of a deposit, the recipient of a
withdrawal.

`risk_score` is `None` until screening finishes. A completed check can have a score
of 0, which is not the same as having no score yet.

## Error handling

Every failure is a `TronzapError`. Match on its variants to handle one kind of
failure:

```text
TronzapError
├── Transport(TransportError)        — no response arrived
│     kind(): Timeout | Connect | Tls | Other
├── Api(ApiError)                    — the API answered with a non-zero code, or a non-2xx status
├── Authentication(ApiError)         — error code 1, or HTTP 401 / 403
├── Validation(ValidationError)      — invalid arguments; nothing was sent
└── Serialization(SerializationError) — a 2xx response the SDK could not read
```

`ApiError` carries the HTTP status (`status()`), the API error code (`code()` and
`error_code()`), the error key, the message, the request ID, the raw response body
(`body()`) and, for HTTP 429, the `Retry-After` delay. `SerializationError` carries
the status and the raw body too.

```rust,no_run
use tronzap_sdk::requests::EnergyTransactionRequest;
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn buy(client: &TronzapClient) {
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000);
    match client.create_energy_transaction(&request).await {
        Ok(tx) => println!("bought, transaction {}", tx.id),
        // Application-level failure: the code says exactly what went wrong.
        Err(TronzapError::Api(e)) => match e.error_code() {
            // The key may narrow it down, e.g. "invalid_tron_address.from_address"
            Some(ErrorCode::InvalidTronAddress) => eprintln!("bad address: {:?}", e.key()),
            Some(ErrorCode::InsufficientFunds) => eprintln!("top up the account"),
            Some(ErrorCode::AddressNotActivated) => eprintln!("activate the address first"),
            // Back off and retry, after e.retry_after() if the API sent it.
            _ if e.is_rate_limited() => eprintln!("rate limited, retry after {:?}", e.retry_after()),
            // Transient; safe to retry.
            _ if e.is_server_error() => eprintln!("server error, retry later"),
            _ => eprintln!("api error {:?}: {} (request {:?})", e.code(), e.message(), e.request_id()),
        },
        // Bad token or signature.
        Err(TronzapError::Authentication(e)) => eprintln!("check the credentials: {e}"),
        // Transient; safe to retry.
        Err(TronzapError::Transport(e)) if e.is_timeout() => eprintln!("timed out"),
        Err(e) => eprintln!("{e}"),
    }
}
```

`request_id()` is the identifier the API assigns to each request. Quote it when
contacting support.

An API error takes precedence over the HTTP status: the API reports some failures
with a 2xx status and others with a 4xx or 5xx status, so a readable payload with
a non-zero code is always reported with that code, never as a bare HTTP error.

### API error codes

| Code | Constant | Description |
|------|----------|-------------|
| 1 | `AuthError` | Authentication error – invalid API token or signature |
| 2 | `InvalidServiceOrParams` | Invalid service or parameters |
| 5 | `WalletNotFound` | Internal wallet not found. Contact support. |
| 6 | `InsufficientFunds` | Insufficient funds |
| 10 | `InvalidTronAddress` | Invalid TRON address |
| 11 | `InvalidEnergyAmount` | Invalid energy amount |
| 12 | `InvalidDuration` | Invalid duration |
| 20 | `TransactionNotFound` | Transaction/subscription not found |
| 21 | `CannotStopSubscription` | Cannot stop subscription |
| 24 | `AddressNotActivated` | Address not activated |
| 25 | `AddressAlreadyActivated` | Address already activated |
| 30 | `AmlCheckNotFound` | AML check not found |
| 35 | `ServiceNotAvailable` | Service not available |
| 50 | `InvalidBandwidthAmount` | Invalid bandwidth amount |
| 500 | `InternalServerError` | Internal server error – contact support |

The constants are variants of the `ErrorCode` enum. For a code this SDK version
does not know, `error_code()` returns `None` and the number is still available from
`code()`.

## Decimal and timestamp fields

Amounts and prices are [`rust_decimal::Decimal`](https://docs.rs/rust_decimal),
re-exported as `tronzap_sdk::models::Decimal`: exact decimal arithmetic, and
`1.50 == 1.5`. The API encodes money as a JSON number in some responses and as a
JSON string in others; both forms are read the same way. For example, the cost of
65000 energy is `rate.price * Decimal::from(65000) / Decimal::from(1000)`.

Timestamps are `Timestamp`: `as_str()` is the text exactly as the API sent it, and
`unix_timestamp()` and `to_system_time()` parse it. The several formats the API
emits are accepted, and times without an offset are read as UTC. An unrecognised
timestamp makes the parsers return `None` instead of failing the whole response.

Values the API may add in the future, such as a new transaction status, are
reported as the `Unknown(String)` variant of the matching enum instead of failing.

## Testing

```bash
cargo test --all-features
```

It runs the unit tests and the integration tests against a local HTTP server: the
exact request body and signature of every endpoint, API and HTTP errors, malformed
JSON, timeouts, network and TLS failures, and concurrent use.

## License

The MIT License (MIT). Please see [License File](LICENSE) for more information.

## Support

For support, please contact [support@tronzap.com](mailto:support@tronzap.com).
