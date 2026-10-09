//! Parameters of the API calls.
//!
//! Required values are arguments of `new` (or of a named constructor) and optional
//! ones are chainable setters, so a request can never miss a required field:
//!
//! ```
//! use tronzap_sdk::requests::EnergyTransactionRequest;
//!
//! let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
//!     .duration(1)
//!     .external_id("order-42")
//!     .activate_address(true);
//! ```
//!
//! Every request is validated before it is sent. An invalid one fails with
//! [`TronzapError::Validation`](crate::TronzapError::Validation) and never reaches
//! the API.

mod aml;
mod subscription;
mod transaction;

use serde::Serialize;

use crate::error::ValidationError;

pub use aml::{AmlCheckRequest, AmlHistoryRequest};
pub use subscription::{StartSubscriptionRequest, SubscriptionHistoryRequest, SubscriptionRequest};
pub use transaction::{
    AddressActivationRequest, BandwidthTransactionRequest, CheckTransactionRequest, EnergyTransactionRequest,
    ResourceBundleTransactionRequest,
};

/// The USDT (TRC20) contract the API estimates against when
/// [`EstimateEnergyRequest::contract_address`] is not set.
pub const USDT_CONTRACT_ADDRESS: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

pub(crate) const DEFAULT_DURATION: u32 = 1;

/// A token transfer to estimate the energy for.
///
/// ```
/// use tronzap_sdk::requests::EstimateEnergyRequest;
///
/// let request = EstimateEnergyRequest::new("TSenderAddress", "TRecipientAddress");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EstimateEnergyRequest {
    from_address: String,
    to_address: String,
    contract_address: Option<String>,
}

impl EstimateEnergyRequest {
    /// Estimates a USDT (TRC20) transfer from one TRON address to another.
    ///
    /// The API does not reject equal sender and recipient addresses, so check them
    /// yourself if that matters.
    pub fn new(from_address: impl Into<String>, to_address: impl Into<String>) -> Self {
        EstimateEnergyRequest {
            from_address: from_address.into(),
            to_address: to_address.into(),
            contract_address: None,
        }
    }

    /// Estimates a transfer of another TRC20 token instead of USDT.
    #[must_use]
    pub fn contract_address(mut self, contract_address: impl Into<String>) -> Self {
        self.contract_address = Some(contract_address.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("from_address", &self.from_address)?;
        required("to_address", &self.to_address)?;
        optional("contract_address", self.contract_address.as_deref())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            from_address: &'a str,
            to_address: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            contract_address: Option<&'a str>,
        }
        Wire {
            from_address: &self.from_address,
            to_address: &self.to_address,
            contract_address: self.contract_address.as_deref(),
        }
    }
}

/// An energy purchase to price without creating a transaction.
///
/// ```
/// use tronzap_sdk::requests::CalculateRequest;
///
/// let request = CalculateRequest::new("TRecipientAddress", 65000).duration(1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculateRequest {
    address: String,
    energy: u64,
    duration: u32,
}

impl CalculateRequest {
    /// Prices `energy` units of energy for `address`, rented for 1 hour.
    pub fn new(address: impl Into<String>, energy: u64) -> Self {
        CalculateRequest { address: address.into(), energy, duration: DEFAULT_DURATION }
    }

    /// Sets the rental duration in hours. Defaults to 1.
    #[must_use]
    pub fn duration(mut self, hours: u32) -> Self {
        self.duration = hours;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("address", &self.address)?;
        positive("energy", self.energy)?;
        positive("duration", u64::from(self.duration))
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            address: &'a str,
            amount: u64,
            duration: u32,
        }
        Wire { address: &self.address, amount: self.energy, duration: self.duration }
    }
}

#[derive(Serialize)]
pub(crate) struct AddressParam<'a> {
    pub(crate) address: &'a str,
}

#[derive(Serialize)]
pub(crate) struct IdParam<'a> {
    pub(crate) id: &'a str,
}

pub(crate) fn required(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() { Err(ValidationError::new(field, "is required")) } else { Ok(()) }
}

pub(crate) fn optional(field: &'static str, value: Option<&str>) -> Result<(), ValidationError> {
    match value {
        Some(v) if v.trim().is_empty() => Err(ValidationError::new(field, "must not be blank when set")),
        _ => Ok(()),
    }
}

pub(crate) fn positive(field: &'static str, value: u64) -> Result<(), ValidationError> {
    if value == 0 { Err(ValidationError::new(field, "must be positive")) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn wire(value: impl Serialize) -> Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn estimate_omits_contract_unless_set() {
        let request = EstimateEnergyRequest::new("TFrom", "TTo");
        assert_eq!(wire(request.wire()), json!({"from_address": "TFrom", "to_address": "TTo"}));
        let request = request.contract_address(USDT_CONTRACT_ADDRESS);
        assert_eq!(wire(request.wire())["contract_address"], USDT_CONTRACT_ADDRESS);
    }

    #[test]
    fn calculate_sends_amount_and_default_duration() {
        let request = CalculateRequest::new("TAddr", 65000);
        assert_eq!(wire(request.wire()), json!({"address": "TAddr", "amount": 65000, "duration": 1}));
    }

    #[test]
    fn validation_names_the_field() {
        let e = EstimateEnergyRequest::new(" ", "TTo").validate().unwrap_err();
        assert_eq!(e.field(), "from_address");
        let e = CalculateRequest::new("TAddr", 0).validate().unwrap_err();
        assert_eq!(e.field(), "energy");
        let e = CalculateRequest::new("TAddr", 65000).duration(0).validate().unwrap_err();
        assert_eq!(e.field(), "duration");
        let e = EstimateEnergyRequest::new("TFrom", "TTo").contract_address("").validate().unwrap_err();
        assert_eq!(e.field(), "contract_address");
    }
}
