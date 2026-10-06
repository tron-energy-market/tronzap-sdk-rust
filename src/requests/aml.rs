use serde::Serialize;

use super::{positive, required};
use crate::error::ValidationError;
use crate::models::{AmlCheckType, AmlDirection, AmlStatus};

/// An address or a transaction hash to screen.
///
/// ```
/// use tronzap_sdk::models::AmlDirection;
/// use tronzap_sdk::requests::AmlCheckRequest;
///
/// let address = AmlCheckRequest::for_address("TRX", "TAddressToScreen");
/// let transaction =
///     AmlCheckRequest::for_hash("BTC", "bc1RecipientAddress", "TX_HASH", AmlDirection::Withdrawal);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmlCheckRequest {
    check_type: AmlCheckType,
    network: String,
    address: String,
    hash: Option<String>,
    direction: Option<AmlDirection>,
}

impl AmlCheckRequest {
    /// Screens a wallet address on `network`, a code such as `TRX`, `BTC` or `ETH`.
    pub fn for_address(network: impl Into<String>, address: impl Into<String>) -> Self {
        AmlCheckRequest {
            check_type: AmlCheckType::Address,
            network: network.into(),
            address: address.into(),
            hash: None,
            direction: None,
        }
    }

    /// Screens the transaction `hash` on `network`.
    ///
    /// `address` is the recipient address of the transaction, where the funds were
    /// received. `direction` says which side of the transaction you are on:
    ///
    /// - [`AmlDirection::Deposit`]: the funds were sent to your address, so
    ///   `address` is your address;
    /// - [`AmlDirection::Withdrawal`]: you sent the funds, so `address` is the
    ///   external recipient's address.
    ///
    /// The risk is scored for the counterparty: the sender of a deposit, the
    /// recipient of a withdrawal.
    pub fn for_hash(
        network: impl Into<String>,
        address: impl Into<String>,
        hash: impl Into<String>,
        direction: AmlDirection,
    ) -> Self {
        AmlCheckRequest {
            check_type: AmlCheckType::Hash,
            network: network.into(),
            address: address.into(),
            hash: Some(hash.into()),
            direction: Some(direction),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("network", &self.network)?;
        required("address", &self.address)?;
        if self.check_type == AmlCheckType::Hash {
            required("hash", self.hash.as_deref().unwrap_or_default())?;
            required("direction", self.direction.as_ref().map(AmlDirection::as_str).unwrap_or_default())?;
        }
        Ok(())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            #[serde(rename = "type")]
            check_type: &'a AmlCheckType,
            network: &'a str,
            address: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            hash: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            direction: Option<&'a AmlDirection>,
        }
        Wire {
            check_type: &self.check_type,
            network: &self.network,
            address: &self.address,
            hash: self.hash.as_deref(),
            direction: self.direction.as_ref(),
        }
    }
}

/// A page of AML check history.
///
/// ```
/// use tronzap_sdk::models::AmlStatus;
/// use tronzap_sdk::requests::AmlHistoryRequest;
///
/// let first_page = AmlHistoryRequest::new();
/// let completed = AmlHistoryRequest::new().page(2).per_page(50).status(AmlStatus::Completed);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmlHistoryRequest {
    page: u32,
    per_page: u32,
    status: Option<AmlStatus>,
}

impl AmlHistoryRequest {
    /// The page size used unless [`AmlHistoryRequest::per_page`] is called.
    pub const DEFAULT_PER_PAGE: u32 = 10;

    /// Returns the first page of all checks, 10 per page.
    pub fn new() -> Self {
        AmlHistoryRequest { page: 1, per_page: Self::DEFAULT_PER_PAGE, status: None }
    }

    /// Sets the page number, starting at 1.
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// Sets the page size. The API accepts 1 to 50.
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = per_page;
        self
    }

    /// Returns only checks in this state.
    #[must_use]
    pub fn status(mut self, status: AmlStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        positive("page", u64::from(self.page))?;
        positive("per_page", u64::from(self.per_page))
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            page: u32,
            per_page: u32,
            #[serde(skip_serializing_if = "Option::is_none")]
            status: Option<&'a AmlStatus>,
        }
        Wire { page: self.page, per_page: self.per_page, status: self.status.as_ref() }
    }
}

impl Default for AmlHistoryRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn wire(value: impl Serialize) -> Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn check_wire_format() {
        assert_eq!(
            wire(AmlCheckRequest::for_address("TRX", "TAddr").wire()),
            json!({"type": "address", "network": "TRX", "address": "TAddr"})
        );
        assert_eq!(
            wire(AmlCheckRequest::for_hash("BTC", "bc1", "HASH", AmlDirection::Withdrawal).wire()),
            json!({"type": "hash", "network": "BTC", "address": "bc1", "hash": "HASH", "direction": "withdrawal"})
        );
    }

    #[test]
    fn history_defaults() {
        assert_eq!(wire(AmlHistoryRequest::default().wire()), json!({"page": 1, "per_page": 10}));
        assert_eq!(
            wire(AmlHistoryRequest::new().page(3).status(AmlStatus::Failed).wire()),
            json!({"page": 3, "per_page": 10, "status": "failed"})
        );
    }

    #[test]
    fn rejects_invalid_values() {
        assert_eq!(AmlCheckRequest::for_address("", "T").validate().unwrap_err().field(), "network");
        assert_eq!(AmlCheckRequest::for_address("TRX", " ").validate().unwrap_err().field(), "address");
        assert_eq!(
            AmlCheckRequest::for_hash("BTC", "bc1", "", AmlDirection::Deposit)
                .validate()
                .unwrap_err()
                .field(),
            "hash"
        );
        let blank = AmlDirection::Unknown(String::new());
        assert_eq!(
            AmlCheckRequest::for_hash("BTC", "bc1", "H", blank).validate().unwrap_err().field(),
            "direction"
        );
        assert_eq!(AmlHistoryRequest::new().page(0).validate().unwrap_err().field(), "page");
        assert_eq!(AmlHistoryRequest::new().per_page(0).validate().unwrap_err().field(), "per_page");
    }
}
