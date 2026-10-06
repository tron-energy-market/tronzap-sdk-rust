# Changelog

All notable changes to this project are documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[1.0.0]: https://github.com/tron-energy-market/tronzap-sdk-rust/releases/tag/v1.0.0
