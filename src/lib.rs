//! The official Rust SDK for the [TronZap](https://tronzap.com/) API.
//!
//! TronZap sells TRON energy and bandwidth, which makes USDT (TRC20) transfers
//! much cheaper. Get API credentials at <https://tronzap.com/> and read the API
//! reference at <https://docs.tronzap.com/>.
//!
//! # Getting started
//!
//! ```no_run
//! use tronzap_sdk::TronzapClient;
//! use tronzap_sdk::requests::{EnergyTransactionRequest, EstimateEnergyRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), tronzap_sdk::TronzapError> {
//!     let client = TronzapClient::builder()
//!         .api_token("your_api_token")
//!         .api_secret("your_api_secret")
//!         .build()?;
//!
//!     let balance = client.get_balance().await?;
//!     println!("balance: {}", balance.balance);
//!
//!     // Estimate how much energy a USDT transfer needs, then buy exactly that much.
//!     let estimate = client
//!         .estimate_energy(&EstimateEnergyRequest::new("TSenderAddress", "TRecipientAddress"))
//!         .await?;
//!     let tx = client
//!         .create_energy_transaction(
//!             &EnergyTransactionRequest::new("TRecipientAddress", estimate.amount).external_id("order-42"),
//!         )
//!         .await?;
//!     println!("transaction {} costs {} and is {}", tx.id, tx.amount, tx.status);
//!     Ok(())
//! }
//! ```
//!
//! # Runtime
//!
//! Requests go through [`reqwest`], so the client needs a [Tokio](https://tokio.rs/)
//! runtime. The crate does not start one itself.
//!
//! # Errors
//!
//! Every call returns [`Result<T, TronzapError>`](Result); nothing panics on a
//! failed request. See [`TronzapError`] for the cases.
//!
//! # Features
//!
//! - `rustls` (default): TLS through rustls, with the platform's certificate
//!   verifier.
//! - `native-tls`: TLS through the platform's library (OpenSSL, Secure Transport,
//!   SChannel). When both TLS features are on, reqwest uses this one.
//! - `system-proxy`: honour the operating system's proxy settings.
//!
//! Without any TLS feature only `http://` base URLs work.

mod client;
mod config;
mod error;
pub mod models;
pub mod requests;
pub mod responses;

pub use client::TronzapClient;
pub use config::{DEFAULT_BASE_URL, DEFAULT_TIMEOUT, TronzapClientBuilder};
pub use error::{
    ApiError, ErrorCode, Result, SerializationError, TransportError, TransportErrorKind, TronzapError,
    ValidationError,
};
pub use reqwest;
pub use rust_decimal;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
#[doc = include_str!("../README.es.md")]
#[doc = include_str!("../README.pt-br.md")]
#[doc = include_str!("../README.ru.md")]
struct Readme;
