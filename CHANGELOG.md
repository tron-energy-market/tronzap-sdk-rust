# Changelog

All notable changes to this project are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2026-11-20

### Added

- Bulk energy orders for up to 10 000 addresses: `TronzapClient::bulk_calculate`, `bulk_create`, `bulk_check`,
  `get_bulk_history` and `bulk_report`, with the requests `BulkItem`, `BulkCalculateRequest`, `BulkCreateRequest`,
  `BulkOrderRequest`, `BulkReportRequest` and `BulkHistoryRequest`, the responses `BulkCalculation`,
  `BulkRejectedRow`, `BulkOrder` and `BulkHistory`, and the `BulkStatus` and `BulkRowStatus` enums. `bulk_report`
  returns the CSV report as a UTF-8 `String`.
- `TronzapClient::verify_webhook_signature` checks the `X-Signature` header of the webhook a bulk order sends when it
  becomes final.
- `ApiError::result`: the `result` the API sends along with an error, as a `serde_json::Value`. Until now the SDK
  dropped it.
- `ApiError::bulk_calculation` holds the new calculation of `BulkPriceChanged` (92), and `ApiError::bulk_rejected`
  the rejected rows of `BulkNoValidItems` (94).
- The error codes `BulkNotFound` (90), `BulkExpired` (91), `BulkPriceChanged` (92), `BulkTooManyItems` (93),
  `BulkNoValidItems` (94), `BulkResourceNotSupported` (95), `PayloadTooLarge` (96) and `RateLimited` (429).
- `tronzap_sdk::serde_json` re-exports the `serde_json` version `ApiError::result` uses.
- The example lists bulk orders, calculates one with an invalid row, checks and reports `TRONZAP_BULK_ID`, and with
  purchases allowed starts a one-address bulk order.

### Changed

- `ApiError::retry_after` also reads a `Retry-After` header in the HTTP date format, and the `retry_after` field of
  the body when the header is missing. It used to read only a header in seconds.
- `ApiError::is_rate_limited` is also `true` for the error code `RateLimited`, not only for HTTP 429.

### Security

- The `rustls` feature requires rustls 0.23.46 or later, which fixes RUSTSEC-2026-0346 to RUSTSEC-2026-0349,
  among them `Debug` output that exposed TLS secrets. A project that already resolved an older 0.23 release gets the
  fix on its next `cargo update`.

## [1.1.0] - 2026-10-09

### Added

- Subscriptions: `TronzapClient::get_subscriptions`, `start_subscription`, `check_subscription`, `stop_subscription`
  and `get_subscription_history`, with the requests `StartSubscriptionRequest`, `SubscriptionRequest` and
  `SubscriptionHistoryRequest`, the responses `SubscriptionPlan`, `Subscription`, `SubscriptionParams` and
  `SubscriptionHistory`, and the `SubscriptionStatus` enum. `get_subscriptions` returns the plans in the order the
  API lists them.

### Changed

- The descriptions of error codes 10 (`InvalidTronAddress`), which also means the address already has an active
  subscription, and 21 (`CannotStopSubscription`), which is returned for a subscription with a transactions limit.

### Deprecated

- `EnergyRate::min_energy` and `EnergyRate::max_energy`: use `min_amount` and `max_amount`.
- `Calculation::energy` and `EnergyEstimate::energy`: use `amount`.

The deprecated fields stay and always equal their replacements: they are now filled from `min_amount`, `max_amount`
and `amount`, and the deprecated fields of the API responses are no longer read.

### Fixed

- `EnergyRate::price` is the price of 1000 units of energy, like `BandwidthRate::price`, not of one unit. The rustdoc,
  the README and the example said otherwise: 65000 energy at a price of 0.03 costs 1.95.
- `DirectRechargeRate::price` is the price of 1000 units of energy as well, not of one unit.

## [1.0.0] - 2026-10-06

First release of the official Rust SDK for the [TronZap API](https://docs.tronzap.com/).

### Added

- `TronzapClient` with every TronZap API operation: services and prices, account balance, address info, energy
  estimates and price calculation, energy, bandwidth, resource bundle and address activation purchases,
  transaction status, direct recharge info, and AML services, checks and history.
- `TronzapClient::builder()` for the API token and secret, base URL, timeout, `User-Agent` and your own
  `reqwest::Client`.
- Typed requests that are validated before sending, and typed responses that accept every encoding the API uses:
  amounts as JSON numbers or strings, timestamps in several formats, empty objects sent as `[]`. Money is
  `rust_decimal::Decimal`, re-exported from the crate.
- `TronzapError` with separate transport, API, authentication, validation and serialization variants, carrying the
  HTTP status, API error code, error key, request ID, `Retry-After` delay and raw response body.
- `rustls` (default), `native-tls` and `system-proxy` features.

[Unreleased]: https://github.com/tron-energy-market/tronzap-sdk-rust/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/tron-energy-market/tronzap-sdk-rust/releases/tag/v1.2.0
[1.1.0]: https://github.com/tron-energy-market/tronzap-sdk-rust/releases/tag/v1.1.0
[1.0.0]: https://github.com/tron-energy-market/tronzap-sdk-rust/releases/tag/v1.0.0
