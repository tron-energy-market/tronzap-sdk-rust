# Покупка энергии Tron через API
## Rust SDK от TronZap.com

[English](README.md) | [Español](README.es.md) | [Português](README.pt-br.md) | **[Русский](README.ru.md)**

[![crates.io](https://img.shields.io/crates/v/tronzap-sdk.svg)](https://crates.io/crates/tronzap-sdk)
[![docs.rs](https://img.shields.io/docsrs/tronzap-sdk)](https://docs.rs/tronzap-sdk)
[![CI](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/tron-energy-market/tronzap-sdk-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Официальный Rust SDK для API TronZap.
Этот SDK позволяет легко интегрировать сервисы TronZap для аренды энергии TRON.

TronZap.com позволяет [покупать энергию TRON](https://tronzap.com/), существенно снижая комиссии при переводах USDT (TRC20).

👉 [Зарегистрируйтесь для получения API ключа](https://tronzap.com), чтобы начать использовать TronZap API и интегрировать его через SDK.

- Сайт: https://tronzap.com/
- Справочник API: https://docs.tronzap.com/
- crates.io: https://crates.io/crates/tronzap-sdk
- Документация: https://docs.rs/tronzap-sdk
- Исходный код: https://github.com/tron-energy-market/tronzap-sdk-rust

## Установка

```toml
[dependencies]
tronzap-sdk = "1.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Требования

- Rust 1.85 или новее
- Runtime [Tokio](https://tokio.rs/): запросы идут через [`reqwest`](https://crates.io/crates/reqwest). Сам SDK runtime не запускает.

## Быстрый старт

```rust,no_run
use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
use tronzap_sdk::TronzapClient;

#[tokio::main]
async fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("ваш_api_token")
        .api_secret("ваш_api_secret")
        .build()?;

    let balance = client.get_balance().await?;
    println!("balance: {} (deposit to {})", balance.balance, balance.address);

    // Оцениваем, сколько энергии нужно для перевода USDT, и покупаем ровно столько.
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

Готовый к запуску обход всех операций лежит в
[`examples/basic_usage.rs`](examples/basic_usage.rs):

```bash
export TRONZAP_API_TOKEN=ваш_api_token
export TRONZAP_API_SECRET=ваш_api_secret
export TRONZAP_BASE_URL=api.tronzap.com   # необязательно
cargo run --example basic_usage
```

По умолчанию пример только читает данные и ничего не тратит. Переменная
`TRONZAP_ALLOW_PURCHASES=1` включает ещё и вызовы, которые создают транзакции и
AML-проверки, а они списывают средства с баланса. Остальные необязательные
переменные описаны в комментарии в начале файла.

## Настройка

Builder принимает два ключа из личного кабинета: API-токен передаётся
как bearer-токен, а API-секрет подписывает тело каждого запроса. Всё остальное
необязательно:

```rust
use std::time::Duration;
use tronzap_sdk::TronzapClient;

fn main() -> Result<(), tronzap_sdk::TronzapError> {
    let client = TronzapClient::builder()
        .api_token("ваш_api_token")
        .api_secret("ваш_api_secret")
        .base_url("api.tronzap.com")        // по умолчанию tronzap_sdk::DEFAULT_BASE_URL
        .timeout(Duration::from_secs(10))   // на запрос; по умолчанию 30 секунд
        .user_agent("my-app/1.0")
        .build()?;
    Ok(())
}
```

`base_url` принимает и голый домен, и полный URL: без схемы подставляется `https`,
завершающий слеш отбрасывается, поэтому `"api.tronzap.com"`, `"api.tronzap.com/"`
и `"https://api.tronzap.com"` равнозначны. Чтобы использовать другую схему,
укажите её явно, например `"http://localhost:8080"` для локального мока.
`TronzapClient::new(token, secret)` — короткий вариант с настройками по умолчанию.

`TronzapClient` дёшево клонируется, не хранит изменяемого состояния и безопасен
для одновременного использования из многих задач, поэтому создайте один клиент на
набор учётных данных и используйте его везде. Его вывод `Debug` никогда не
показывает токен и секрет.

### Свой клиент reqwest

Для прокси, своих сертификатов или настроек пула соединений передайте
`reqwest::Client`. Берите `reqwest` из реэкспорта SDK, чтобы версии всегда
совпадали:

```rust
use tronzap_sdk::{reqwest, TronzapClient};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = reqwest::Client::builder()
        .proxy(reqwest::Proxy::https("http://proxy.internal:3128")?)
        .build()?;

    let client = TronzapClient::builder()
        .api_token("ваш_api_token")
        .api_secret("ваш_api_secret")
        .http_client(http)
        .build()?;
    Ok(())
}
```

Клиент используется как есть: заголовки задаются для каждого запроса. Его
собственный таймаут продолжает действовать, если не вызвать ещё и `.timeout(...)`,
который его заменяет.

### Features

| Feature | По умолчанию | Описание |
|---|---|---|
| `rustls` | да | TLS через rustls с проверкой сертификатов средствами платформы |
| `native-tls` | нет | TLS через системную библиотеку (OpenSSL, Secure Transport, SChannel). Если включены обе TLS-feature, используется эта |
| `system-proxy` | нет | Учитывать системные настройки прокси |

```toml
tronzap-sdk = { version = "1.1", default-features = false, features = ["native-tls"] }
```

## Доступные методы

| Метод | Endpoint | Описание |
|---|---|---|
| `get_services()` | `/v1/services` | Доступные сервисы и цены |
| `get_balance()` | `/v1/balance` | Текущий баланс аккаунта |
| `get_address_info(address)` | `/v1/address-info` | Ресурсы адреса (energy, bandwidth) и балансы (TRX, USDT) |
| `estimate_energy(&request)` | `/v1/estimate-energy` | Сколько энергии нужно для перевода и сколько она стоит |
| `calculate(&request)` | `/v1/calculate` | Цена покупки без создания транзакции |
| `create_energy_transaction(&request)` | `/v1/transaction/new` | Купить энергию |
| `create_bandwidth_transaction(&request)` | `/v1/transaction/new` | Купить bandwidth |
| `create_resource_bundle_transaction(&request)` | `/v1/transaction/new` | Купить энергию и bandwidth одной транзакцией |
| `create_address_activation_transaction(&request)` | `/v1/transaction/new` | Активировать адрес TRON |
| `check_transaction(&request)` | `/v1/transaction/check` | Статус транзакции по id или external id |
| `get_direct_recharge_info()` | `/v1/direct-recharge-info` | Адрес и тарифы прямого пополнения |
| `get_aml_services()` | `/v1/aml-checks` | AML-сервисы и цены |
| `create_aml_check(&request)` | `/v1/aml-checks/new` | Запустить AML-проверку |
| `check_aml_status(id)` | `/v1/aml-checks/check` | Статус и результат AML-проверки |
| `get_aml_history(&request)` | `/v1/aml-checks/history` | История AML-проверок постранично |
| `get_subscriptions()` | `/v1/subscriptions` | Планы подписок и цены |
| `start_subscription(&request)` | `/v1/subscription/start` | Подписать адрес на план |
| `check_subscription(&request)` | `/v1/subscription/check` | Статус подписки по id или внешнему id |
| `stop_subscription(&request)` | `/v1/subscription/stop` | Остановить подписку |
| `get_subscription_history(&request)` | `/v1/subscriptions/history` | История подписок постранично |
| `request(endpoint, &params)` | любой | Подписанный вызов endpoint, который SDK ещё не оборачивает |

Каждый метод `async` и возвращает `Result<T, TronzapError>`. Если отбросить
future, запрос отменяется.

Параметры задаются типами запросов из `tronzap_sdk::requests`. Обязательные
значения — аргументы `new` (или именованного конструктора, например
`CheckTransactionRequest::by_external_id`), необязательные — методы-сеттеры в
цепочке. Запрос проверяется до отправки, поэтому невалидный возвращает
`TronzapError::Validation` и до API не доходит. Значения по умолчанию совпадают с
API: `duration` — 1 час, история AML и подписок начинается со
страницы 1 по 10 записей. Исключение — `StartSubscriptionRequest`: нулевые
`duration_days` и `transactions_limit` означают отсутствие ограничения.

Результаты — структуры из `tronzap_sdk::responses`. Списки никогда не
отсутствуют, а значения, которые API может опустить, имеют тип `Option`.

### Покупка ресурсов

```rust,no_run
use tronzap_sdk::requests::{
    AddressActivationRequest, BandwidthTransactionRequest, EnergyTransactionRequest,
    ResourceBundleTransactionRequest,
};
use tronzap_sdk::TronzapClient;

async fn buy(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    // Энергия, при желании с активацией адреса в том же вызове.
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
        .duration(1) // часы; доступные длительности смотрите в get_services()
        .external_id("order-42")
        .activate_address(true);
    client.create_energy_transaction(&request).await?;

    // Bandwidth.
    let request = BandwidthTransactionRequest::new("TRecipientAddress", 345).external_id("bandwidth-1");
    client.create_bandwidth_transaction(&request).await?;

    // Энергия и bandwidth вместе, одной транзакцией.
    let request = ResourceBundleTransactionRequest::new("TRecipientAddress", 65000, 345).external_id("bundle-1");
    client.create_resource_bundle_transaction(&request).await?;

    // Только активация.
    let request = AddressActivationRequest::new("TRecipientAddress").external_id("activation-1");
    client.create_address_activation_transaction(&request).await?;
    Ok(())
}
```

Цены энергии и bandwidth указаны за 1000 единиц: в `get_services()` 65000 энергии
при `EnergyRate::price`, равном 0.03, стоят 0.03 × 65000 / 1000 = 1.95, а 345
bandwidth при `BandwidthRate::price`, равном 1, стоят 0.345.

Сейчас API возвращает resource bundle с `service`, равным `Service::Energy`, а не
`Service::ResourceBundle`. Какие ресурсы входят в транзакцию, смотрите в
`params.amounts`.

### Отслеживание транзакции

Транзакция проходит путь `New` → `Pending` → `Success` или `Failed`:

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

### AML-проверка

```rust,no_run
use tronzap_sdk::models::AmlStatus;
use tronzap_sdk::requests::AmlCheckRequest;
use tronzap_sdk::TronzapClient;

async fn screen(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    let check = client.create_aml_check(&AmlCheckRequest::for_address("TRX", "TAddressToScreen")).await?;
    // или AmlCheckRequest::for_hash("BTC", "bc1RecipientAddress", "TX_HASH", AmlDirection::Withdrawal)

    let result = client.check_aml_status(&check.id).await?;
    if result.status == AmlStatus::Completed {
        println!("{:?} {:?} {} factor(s)", result.risk_level, result.risk_score, result.risk_factors.len());
    }
    Ok(())
}
```

Для проверки по hash `address` — это адрес получателя средств в транзакции, а
direction указывает, на какой стороне транзакции вы: `Deposit`, если средства
пришли на ваш адрес (`address` — ваш адрес), `Withdrawal`, если их отправили вы
(`address` — адрес внешнего получателя). Риск оценивается для контрагента: для
deposit — для отправителя, для withdrawal — для получателя.

`risk_score` равен `None`, пока проверка не завершена. У завершённой проверки
score может быть 0, и это не то же самое, что отсутствие score.

### Подписки

Подписка обеспечивает адрес энергией для каждой транзакции, пока её не
остановят или не закончатся её дни или транзакции. Выберите план из
`get_subscriptions` и передайте его `subscription_id`, например
`"unlimited_energy"`, а не числовой `id`:

```rust,no_run
use tronzap_sdk::models::SubscriptionStatus;
use tronzap_sdk::requests::{StartSubscriptionRequest, SubscriptionHistoryRequest, SubscriptionRequest};
use tronzap_sdk::TronzapClient;

async fn subscribe(client: &TronzapClient) -> tronzap_sdk::Result<()> {
    for plan in client.get_subscriptions().await? {
        println!("{} {} {}", plan.subscription_id, plan.initial_price, plan.price);
    }

    let request = StartSubscriptionRequest::new("unlimited_energy", "TRecipientAddress")
        .duration_days(30) // 0 — без ограничения по времени
        .transactions_limit(0) // 0 — без лимита
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

Запуск, проверка и остановка возвращают подписку с её `params`, а история
вместо них — счётчики использования `transactions_used`, `energy_used` и
`total_price`, а `params` равен `None`. Подписку с лимитом транзакций остановить
нельзя (`ErrorCode::CannotStopSubscription`).

## Обработка ошибок

Любой сбой — это `TronzapError`. Сопоставляйте варианты, чтобы обработать
конкретный вид сбоя:

```text
TronzapError
├── Transport(TransportError)        — ответ не пришёл
│     kind(): Timeout | Connect | Tls | Other
├── Api(ApiError)                    — API ответил ненулевым code или статусом не 2xx
├── Authentication(ApiError)         — код ошибки 1 или HTTP 401 / 403
├── Validation(ValidationError)      — невалидные аргументы; ничего не отправлено
└── Serialization(SerializationError) — ответ 2xx, который SDK не смог прочитать
```

`ApiError` содержит HTTP-статус (`status()`), код ошибки API (`code()` и
`error_code()`), ключ ошибки, сообщение, ID запроса, сырое тело ответа (`body()`)
и, для HTTP 429, задержку из `Retry-After`. `SerializationError` тоже содержит
статус и сырое тело.

```rust,no_run
use tronzap_sdk::requests::EnergyTransactionRequest;
use tronzap_sdk::{ErrorCode, TronzapClient, TronzapError};

async fn buy(client: &TronzapClient) {
    let request = EnergyTransactionRequest::new("TRecipientAddress", 65000);
    match client.create_energy_transaction(&request).await {
        Ok(tx) => println!("bought, transaction {}", tx.id),
        // Сбой на уровне приложения: код точно говорит, что пошло не так.
        Err(TronzapError::Api(e)) => match e.error_code() {
            // Ключ может уточнить причину, например "invalid_tron_address.from_address"
            Some(ErrorCode::InvalidTronAddress) => eprintln!("bad address: {:?}", e.key()),
            Some(ErrorCode::InsufficientFunds) => eprintln!("top up the account"),
            Some(ErrorCode::AddressNotActivated) => eprintln!("activate the address first"),
            // Подождите и повторите, после e.retry_after(), если API его прислал.
            _ if e.is_rate_limited() => eprintln!("rate limited, retry after {:?}", e.retry_after()),
            // Временный сбой; можно повторить.
            _ if e.is_server_error() => eprintln!("server error, retry later"),
            _ => eprintln!("api error {:?}: {} (request {:?})", e.code(), e.message(), e.request_id()),
        },
        // Неверный токен или подпись.
        Err(TronzapError::Authentication(e)) => eprintln!("check the credentials: {e}"),
        // Временный сбой; можно повторить.
        Err(TronzapError::Transport(e)) if e.is_timeout() => eprintln!("timed out"),
        Err(e) => eprintln!("{e}"),
    }
}
```

`request_id()` — идентификатор, который API присваивает каждому запросу.
Указывайте его при обращении в поддержку.

Ошибка API важнее HTTP-статуса: часть сбоев API возвращает со статусом 2xx, а
часть — с 4xx или 5xx, поэтому читаемый payload с ненулевым кодом всегда
сообщается с этим кодом, а не как голая HTTP-ошибка.

### Коды ошибок API

| Код | Константа | Описание |
|------|----------|-------------|
| 1 | `AuthError` | Ошибка аутентификации: неверный API-токен или подпись |
| 2 | `InvalidServiceOrParams` | Неверный сервис или параметры |
| 5 | `WalletNotFound` | Внутренний кошелёк не найден. Обратитесь в поддержку. |
| 6 | `InsufficientFunds` | Недостаточно средств |
| 10 | `InvalidTronAddress` | Неверный адрес TRON, или у адреса уже есть активная подписка |
| 11 | `InvalidEnergyAmount` | Неверное количество энергии |
| 12 | `InvalidDuration` | Неверная длительность |
| 20 | `TransactionNotFound` | Транзакция/подписка не найдена |
| 21 | `CannotStopSubscription` | Невозможно остановить подписку, например, у неё есть лимит транзакций |
| 24 | `AddressNotActivated` | Адрес не активирован |
| 25 | `AddressAlreadyActivated` | Адрес уже активирован |
| 30 | `AmlCheckNotFound` | AML-проверка не найдена |
| 35 | `ServiceNotAvailable` | Сервис недоступен |
| 50 | `InvalidBandwidthAmount` | Неверное количество bandwidth |
| 500 | `InternalServerError` | Внутренняя ошибка сервера: обратитесь в поддержку |

Константы — варианты enum `ErrorCode`. Для кода, который эта версия SDK не знает,
`error_code()` возвращает `None`, а само число по-прежнему доступно через
`code()`.

## Числовые поля и даты

Суммы и цены имеют тип [`rust_decimal::Decimal`](https://docs.rs/rust_decimal),
реэкспортированный как `tronzap_sdk::models::Decimal`: точная десятичная
арифметика и `1.50 == 1.5`. API кодирует деньги в одних ответах как JSON-число, а
в других как JSON-строку; обе формы читаются одинаково. Например, стоимость 65000
энергии — это `rate.price * Decimal::from(65000) / Decimal::from(1000)`.

Даты имеют тип `Timestamp`: `as_str()` — текст в точности как прислал API, а
`unix_timestamp()` и `to_system_time()` его разбирают. Принимаются все форматы,
которые использует API, а время без смещения читается как UTC. Для
нераспознанной даты парсеры возвращают `None`, а не роняют весь ответ.

Значения, которые API может добавить в будущем, например новый статус
транзакции, приходят как вариант `Unknown(String)` соответствующего enum, а не
вызывают ошибку.

## Тестирование

```bash
cargo test --all-features
```

Запускаются unit-тесты и интеграционные тесты против локального HTTP-сервера:
точное тело запроса и подпись каждого endpoint, ошибки API и HTTP, битый JSON,
таймауты, сетевые и TLS-сбои, а также одновременное использование.

## Лицензия

Лицензия MIT (MIT). Подробнее в [файле лицензии](LICENSE).

## Поддержка

По вопросам поддержки пишите на [support@tronzap.com](mailto:support@tronzap.com).
