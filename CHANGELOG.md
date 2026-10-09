# Changelog

All notable changes to this project are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/tron-energy-market/tronzap-sdk-rust/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/tron-energy-market/tronzap-sdk-rust/releases/tag/v1.0.0
