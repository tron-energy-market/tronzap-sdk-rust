# Alquiler de Energía Tron vía API
## SDK Rust por TronZap.com

[English](README.md) | **[Español](README.es.md)** | [Português](README.pt-br.md) | [Русский](README.ru.md)

[![crates.io](https://img.shields.io/crates/v/tronzap-sdk.svg)](https://crates.io/crates/tronzap-sdk)
[![docs.rs](https://img.shields.io/docsrs/tronzap-sdk)](https://docs.rs/tronzap-sdk)
[![CI](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

SDK oficial en Rust para la API de TronZap.
Este SDK permite integrar fácilmente los servicios de TronZap para alquilar energía TRON.

TronZap.com permite [comprar energía TRON](https://tronzap.com/), reduciendo significativamente las comisiones en transferencias de USDT (TRC20).

👉 [Regístrate para obtener una clave API](https://tronzap.com) para comenzar a usar la API de TronZap e integrarla a través del SDK.

- Sitio web: https://tronzap.com/
- Referencia de la API: https://docs.tronzap.com/
- crates.io: https://crates.io/crates/tronzap-sdk
- Documentación: https://docs.rs/tronzap-sdk
- Código fuente: https://github.com/tron-energy-market/tronzap-sdk-rust

## Instalación

```toml
[dependencies]
tronzap-sdk = "1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Requisitos

- Rust 1.85 o posterior
- Un runtime de [Tokio](https://tokio.rs/): las solicitudes pasan por [`reqwest`](https://crates.io/crates/reqwest). El SDK no inicia un runtime por sí mismo.

## Inicio rápido

```rust,no_run
use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
use tronzap_sdk::TronzapClient;

#[tokio::main]
async fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("su_api_token")
        .api_secret("su_api_secret")
        .build()?;

    let balance = client.get_balance().await?;
    println!("balance: {} (deposit to {})", balance.balance, balance.address);

    // Estimamos cuánta energía necesita una transferencia de USDT y compramos exactamente esa cantidad.
    let estimate = client
        .estimate_energy(&EstimateEnergyRequest::new("TSenderAddress", "TRecipientAddress"))
        .await?;

    let request = EnergyTransactionRequest::new("TRecipientAddress", estimate.energy)
        .duration(1)
        .external_id("order-42")
        .activate_address(true);
    let tx = client.create_energy_transaction(&request).await?;
    println!("transaction {} costs {} and is {}", tx.id, tx.amount, tx.status);
    Ok(())
}
```

Un recorrido ejecutable por todas las operaciones está en
[`examples/basic_usage.rs`](examples/basic_usage.rs):

```bash
export TRONZAP_API_TOKEN=su_api_token
export TRONZAP_API_SECRET=su_api_secret
export TRONZAP_BASE_URL=api.tronzap.com   # opcional
cargo run --example basic_usage
```

Por defecto solo lee y no gasta nada. Con `TRONZAP_ALLOW_PURCHASES=1` también
ejecuta los endpoints que crean transacciones y verificaciones AML, que descuentan
del saldo de la cuenta. Las demás variables opcionales se describen en el
comentario al inicio del archivo.

## Configuración

El builder recibe las dos credenciales de tu panel: el token de la API se envía
como bearer token y el secreto de la API firma el cuerpo de cada solicitud. Todo
lo demás es opcional:

```rust
use std::time::Duration;
use tronzap_sdk::TronzapClient;

fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("su_api_token")
        .api_secret("su_api_secret")
        .base_url("api.tronzap.com")        // por defecto tronzap_sdk::DEFAULT_BASE_URL
        .timeout(Duration::from_secs(10))   // por solicitud; por defecto 30 segundos
        .user_agent("my-app/1.0")
        .build()?;
    Ok(())
}
```

`base_url` acepta un dominio sin esquema o una URL completa: si falta el esquema se
usa `https` y se elimina la barra final, así que `"api.tronzap.com"`,
`"api.tronzap.com/"` y `"https://api.tronzap.com"` son equivalentes. Indica un
esquema explícito para evitarlo, por ejemplo `"http://localhost:8080"` con un mock
local. `TronzapClient::new(token, secret)` es un atajo con los valores por defecto.

Un `TronzapClient` es barato de clonar, no guarda estado mutable y se puede usar
desde muchas tareas a la vez, así que crea uno por cada juego de credenciales y
compártelo. Su salida `Debug` nunca muestra el token ni el secreto.

### Tu propio cliente reqwest

Para usar un proxy, certificados propios o ajustes del pool de conexiones, pasa un
`reqwest::Client`. Toma `reqwest` del reexport del SDK para que las versiones
coincidan siempre:

```rust
use tronzap_sdk::{reqwest, TronzapClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https("http://proxy.internal:3128")?)
        .build()?;

    let client = TronzapClient::builder()
        .api_token("su_api_token")
        .api_secret("su_api_secret")
        .http_client(http)
        .build()?;
    Ok(())
}
```

El cliente se usa tal cual: las cabeceras se fijan en cada solicitud. Su propio
timeout sigue vigente salvo que también llames a `.timeout(...)`, que lo reemplaza.

### Features

| Feature | Por defecto | Descripción |
|---|---|---|
| `rustls` | sí | TLS con rustls y el verificador de certificados de la plataforma |
| `native-tls` | no | TLS con la biblioteca de la plataforma (OpenSSL, Secure Transport, SChannel). Con ambas features de TLS activas se usa esta |
| `system-proxy` | no | Respetar la configuración de proxy del sistema operativo |

```toml
tronzap-sdk = { version = "1.0", default-features = false, features = ["native-tls"] }
```

## Métodos disponibles

| Método | Endpoint | Descripción |
|---|---|---|
| `get_services()` | `/v1/services` | Servicios disponibles y precios |
| `get_balance()` | `/v1/balance` | Saldo actual de la cuenta |
| `get_address_info(address)` | `/v1/address-info` | Recursos de la dirección (energy, bandwidth) y saldos (TRX, USDT) |
| `estimate_energy(&request)` | `/v1/estimate-energy` | Energía que necesita una transferencia y su costo |
| `calculate(&request)` | `/v1/calculate` | Precio de una compra sin crear una transacción |
| `create_energy_transaction(&request)` | `/v1/transaction/new` | Comprar energía |
| `create_bandwidth_transaction(&request)` | `/v1/transaction/new` | Comprar bandwidth |
| `create_resource_bundle_transaction(&request)` | `/v1/transaction/new` | Comprar energía y bandwidth en una sola transacción |
| `create_address_activation_transaction(&request)` | `/v1/transaction/new` | Activar una dirección TRON |
| `check_transaction(&request)` | `/v1/transaction/check` | Estado de una transacción, por id o external id |
| `get_direct_recharge_info()` | `/v1/direct-recharge-info` | Dirección y tarifas de recarga directa |
| `get_aml_services()` | `/v1/aml-checks` | Servicios AML y precios |
| `create_aml_check(&request)` | `/v1/aml-checks/new` | Iniciar una verificación AML |
| `check_aml_status(id)` | `/v1/aml-checks/check` | Estado y resultado de una verificación AML |
| `get_aml_history(&request)` | `/v1/aml-checks/history` | Historial paginado de verificaciones AML |
| `request(endpoint, &params)` | cualquiera | Llamada firmada a un endpoint que el SDK aún no cubre |

Cada método es `async` y devuelve `Result<T, TronzapError>`. Descartar el future
cancela la solicitud.

Los parámetros están en los tipos de solicitud de `tronzap_sdk::requests`. Los
valores obligatorios son argumentos de `new` (o de un constructor con nombre, como
`CheckTransactionRequest::by_external_id`) y los opcionales son setters
encadenables. La solicitud se valida antes de enviarse, así que una inválida
devuelve `TronzapError::Validation` y nunca llega a la API. Los valores por
defecto coinciden con la API: `duration` es 1 hora y el historial AML empieza en
la página 1 con 10 elementos.

Los resultados son structs de `tronzap_sdk::responses`. Las listas nunca faltan, y
los valores que la API puede omitir son `Option`.

### Comprar recursos

```rust,no_run
use tronzap_sdk::requests::{
    AddressActivationRequest, BandwidthTransactionRequest, EnergyTransactionRequest,
    ResourceBundleTransactionRequest,
};
use tronzap_sdk::TronzapClient;

async fn buy(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    // Energía, opcionalmente activando la dirección en la misma llamada.
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
        .duration(1) // horas; consulta get_services() para las duraciones disponibles
        .external_id("order-42")
        .activate_address(true);
    client.create_energy_transaction(&request).await?;

    // Bandwidth.
    let request = BandwidthTransactionRequest::new("TRecipientAddress", 345).external_id("bandwidth-1");
    client.create_bandwidth_transaction(&request).await?;

    // Energía y bandwidth juntos en una sola transacción.
    let request = ResourceBundleTransactionRequest::new("TRecipientAddress", 65000, 345).external_id("bundle-1");
    client.create_resource_bundle_transaction(&request).await?;

    // Solo la activación.
    let request = AddressActivationRequest::new("TRecipientAddress").external_id("activation-1");
    client.create_address_activation_transaction(&request).await?;
    Ok(())
}
```

El precio de la energía es por unidad y el del bandwidth es por 1000 unidades: en
`get_services()`, `EnergyRate::price` × 65000 es el costo de 65000 de energía,
mientras que 345 de bandwidth con un `BandwidthRate::price` de 1 cuestan 0.345.

Actualmente la API devuelve un resource bundle con `service` igual a
`Service::Energy`, no `Service::ResourceBundle`. Lee `params.amounts` para saber
qué recursos contiene una transacción.

### Seguir una transacción

Una transacción pasa por `New` → `Pending` → `Success` o `Failed`:

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

### Verificación AML

```rust,no_run
use tronzap_sdk::models::AmlStatus;
use tronzap_sdk::requests::AmlCheckRequest;
use tronzap_sdk::TronzapClient;

async fn screen(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let check = client.create_aml_check(&AmlCheckRequest::for_address("TRX", "TAddressToScreen")).await?;
    // o AmlCheckRequest::for_hash("BTC", "bc1RecipientAddress", "TX_HASH", AmlDirection::Withdrawal)

    let result = client.check_aml_status(&check.id).await?;
    if result.status == AmlStatus::Completed {
        println!("{:?} {:?} {} factor(s)", result.risk_level, result.risk_score, result.risk_factors.len());
    }
    Ok(())
}
```

En una verificación por hash, `address` es la dirección del destinatario de la
transacción, donde se recibieron los fondos, y la dirección indica en qué lado
estás: `Deposit` si los fondos llegaron a tu dirección (`address` es tu
dirección), `Withdrawal` si los enviaste tú (`address` es la dirección del
destinatario externo). El riesgo se calcula para la contraparte: el remitente en
un deposit, el destinatario en un withdrawal.

`risk_score` es `None` hasta que termina la verificación. Una verificación
completada puede tener un score de 0, que no es lo mismo que no tener score
todavía.

## Gestión de errores

Toda falla es un `TronzapError`. Haz match sobre sus variantes para tratar un
tipo concreto de falla:

```text
TronzapError
├── Transport(TransportError)        — no llegó ninguna respuesta
│     kind(): Timeout | Connect | Tls | Other
├── Api(ApiError)                    — la API respondió con un code distinto de cero o un estado que no es 2xx
├── Authentication(ApiError)         — código de error 1, o HTTP 401 / 403
├── Validation(ValidationError)      — argumentos inválidos; no se envió nada
└── Serialization(SerializationError) — respuesta 2xx que el SDK no pudo leer
```

`ApiError` incluye el estado HTTP (`status()`), el código de error de la API
(`code()` y `error_code()`), la clave del error, el mensaje, el ID de la
solicitud, el cuerpo bruto de la respuesta (`body()`) y, para HTTP 429, el retraso
de `Retry-After`. `SerializationError` también incluye el estado y el cuerpo
bruto.

```rust,no_run
use tronzap_sdk::requests::EnergyTransactionRequest;
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn buy(client: &TronzapClient) {
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000);
    match client.create_energy_transaction(&request).await {
        Ok(tx) => println!("bought, transaction {}", tx.id),
        // Falla a nivel de aplicación: el código indica exactamente qué salió mal.
        Err(TronzapError::Api(e)) => match e.error_code() {
            // La clave puede precisarlo, p. ej. "invalid_tron_address.from_address"
            Some(ErrorCode::InvalidTronAddress) => eprintln!("bad address: {:?}", e.key()),
            Some(ErrorCode::InsufficientFunds) => eprintln!("top up the account"),
            Some(ErrorCode::AddressNotActivated) => eprintln!("activate the address first"),
            // Espera y reintenta, tras e.retry_after() si la API lo envió.
            _ if e.is_rate_limited() => eprintln!("rate limited, retry after {:?}", e.retry_after()),
            // Transitorio; se puede reintentar.
            _ if e.is_server_error() => eprintln!("server error, retry later"),
            _ => eprintln!("api error {:?}: {} (request {:?})", e.code(), e.message(), e.request_id()),
        },
        // Token o firma incorrectos.
        Err(TronzapError::Authentication(e)) => eprintln!("check the credentials: {e}"),
        // Transitorio; se puede reintentar.
        Err(TronzapError::Transport(e)) if e.is_timeout() => eprintln!("timed out"),
        Err(e) => eprintln!("{e}"),
    }
}
```

`request_id()` es el identificador que la API asigna a cada solicitud.
Indícalo al contactar con soporte.

Un error de la API tiene prioridad sobre el estado HTTP: la API informa algunas
fallas con estado 2xx y otras con 4xx o 5xx, así que un payload legible con un
código distinto de cero siempre se informa con ese código, nunca como un simple
error HTTP.

### Códigos de error de la API

| Código | Constante | Descripción |
|------|----------|-------------|
| 1 | `AuthError` | Error de autenticación: token de API o firma inválidos |
| 2 | `InvalidServiceOrParams` | Servicio o parámetros inválidos |
| 5 | `WalletNotFound` | Billetera interna no encontrada. Contacta con soporte. |
| 6 | `InsufficientFunds` | Fondos insuficientes |
| 10 | `InvalidTronAddress` | Dirección TRON inválida |
| 11 | `InvalidEnergyAmount` | Cantidad de energía inválida |
| 12 | `InvalidDuration` | Duración inválida |
| 20 | `TransactionNotFound` | Transacción/suscripción no encontrada |
| 21 | `CannotStopSubscription` | No se puede detener la suscripción |
| 24 | `AddressNotActivated` | Dirección no activada |
| 25 | `AddressAlreadyActivated` | Dirección ya activada |
| 30 | `AmlCheckNotFound` | Verificación AML no encontrada |
| 35 | `ServiceNotAvailable` | Servicio no disponible |
| 50 | `InvalidBandwidthAmount` | Cantidad de bandwidth inválida |
| 500 | `InternalServerError` | Error interno del servidor: contacta con soporte |

Las constantes son variantes del enum `ErrorCode`. Para un código que esta versión
del SDK no conoce, `error_code()` devuelve `None` y el número sigue disponible en
`code()`.

## Campos decimales y de fecha

Los importes y precios son [`rust_decimal::Decimal`](https://docs.rs/rust_decimal),
reexportado como `tronzap_sdk::models::Decimal`: aritmética decimal exacta y
`1.50 == 1.5`. La API codifica el dinero como número JSON en algunas respuestas y
como cadena JSON en otras; ambas formas se leen igual. Por ejemplo, el costo de
65000 de energía es `rate.price * Decimal::from(65000)`.

Las fechas son `Timestamp`: `as_str()` es el texto exactamente como lo envió la
API, y `unix_timestamp()` y `to_system_time()` lo interpretan. Se aceptan los
distintos formatos que usa la API, y las horas sin desfase se leen como UTC. Ante
una fecha no reconocida los parsers devuelven `None` en lugar de hacer fallar toda
la respuesta.

Los valores que la API pueda añadir en el futuro, como un nuevo estado de
transacción, se informan como la variante `Unknown(String)` del enum
correspondiente en lugar de fallar.

## Pruebas

```bash
cargo test --all-features
```

Ejecuta las pruebas unitarias y las de integración contra un servidor HTTP local:
el cuerpo exacto y la firma de cada endpoint, errores de API y HTTP, JSON mal
formado, timeouts, fallas de red y de TLS, y uso concurrente.

## Licencia

Licencia MIT (MIT). Consulta el [archivo de licencia](LICENSE) para más información.

## Soporte

Para soporte, contacta con [support@tronzap.com](mailto:support@tronzap.com).
