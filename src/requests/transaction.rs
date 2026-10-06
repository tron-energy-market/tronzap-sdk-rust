use serde::Serialize;

use super::{DEFAULT_DURATION, optional, positive, required};
use crate::error::ValidationError;
use crate::models::Service;

/// An energy purchase.
///
/// ```
/// use tronzap_sdk::requests::EnergyTransactionRequest;
///
/// let request = EnergyTransactionRequest::new("TRecipientAddress", 65000)
///     .external_id("order-42")
///     .activate_address(true);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyTransactionRequest {
    address: String,
    energy: u64,
    duration: u32,
    external_id: Option<String>,
    activate_address: bool,
}

impl EnergyTransactionRequest {
    /// Buys `energy` units of energy for `address`, rented for 1 hour.
    pub fn new(address: impl Into<String>, energy: u64) -> Self {
        EnergyTransactionRequest {
            address: address.into(),
            energy,
            duration: DEFAULT_DURATION,
            external_id: None,
            activate_address: false,
        }
    }

    /// Sets the rental duration in hours, one of the durations
    /// [`get_services`](crate::TronzapClient::get_services) lists. Defaults to 1.
    #[must_use]
    pub fn duration(mut self, hours: u32) -> Self {
        self.duration = hours;
        self
    }

    /// Sets your own identifier for the transaction, to look it up later with
    /// [`CheckTransactionRequest::by_external_id`].
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Also activates the address in the same transaction when it is not active yet.
    #[must_use]
    pub fn activate_address(mut self, activate: bool) -> Self {
        self.activate_address = activate;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("address", &self.address)?;
        positive("energy", self.energy)?;
        positive("duration", u64::from(self.duration))?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> NewTransaction<'_> {
        NewTransaction {
            service: Service::Energy,
            params: Params {
                address: &self.address,
                amounts: Some(Amounts { energy: Some(self.energy), bandwidth: None }),
                duration: Some(self.duration),
                activate_address: self.activate_address,
            },
            external_id: self.external_id.as_deref(),
        }
    }
}

/// A bandwidth purchase.
///
/// ```
/// use tronzap_sdk::requests::BandwidthTransactionRequest;
///
/// let request = BandwidthTransactionRequest::new("TRecipientAddress", 345);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandwidthTransactionRequest {
    address: String,
    bandwidth: u64,
    external_id: Option<String>,
}

impl BandwidthTransactionRequest {
    /// Buys `bandwidth` units of bandwidth for `address`.
    pub fn new(address: impl Into<String>, bandwidth: u64) -> Self {
        BandwidthTransactionRequest { address: address.into(), bandwidth, external_id: None }
    }

    /// Sets your own identifier for the transaction.
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("address", &self.address)?;
        positive("bandwidth", self.bandwidth)?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> NewTransaction<'_> {
        NewTransaction {
            service: Service::Bandwidth,
            params: Params {
                address: &self.address,
                amounts: Some(Amounts { energy: None, bandwidth: Some(self.bandwidth) }),
                duration: Some(DEFAULT_DURATION),
                activate_address: false,
            },
            external_id: self.external_id.as_deref(),
        }
    }
}

/// An energy and bandwidth purchase in one transaction.
///
/// ```
/// use tronzap_sdk::requests::ResourceBundleTransactionRequest;
///
/// let request = ResourceBundleTransactionRequest::new("TRecipientAddress", 65000, 345);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBundleTransactionRequest {
    address: String,
    energy: u64,
    bandwidth: u64,
    duration: u32,
    external_id: Option<String>,
    activate_address: bool,
}

impl ResourceBundleTransactionRequest {
    /// Buys `energy` units of energy and `bandwidth` units of bandwidth for
    /// `address`, rented for 1 hour.
    pub fn new(address: impl Into<String>, energy: u64, bandwidth: u64) -> Self {
        ResourceBundleTransactionRequest {
            address: address.into(),
            energy,
            bandwidth,
            duration: DEFAULT_DURATION,
            external_id: None,
            activate_address: false,
        }
    }

    /// Sets the rental duration in hours. Defaults to 1.
    #[must_use]
    pub fn duration(mut self, hours: u32) -> Self {
        self.duration = hours;
        self
    }

    /// Sets your own identifier for the transaction.
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Also activates the address in the same transaction when it is not active yet.
    #[must_use]
    pub fn activate_address(mut self, activate: bool) -> Self {
        self.activate_address = activate;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("address", &self.address)?;
        positive("energy", self.energy)?;
        positive("bandwidth", self.bandwidth)?;
        positive("duration", u64::from(self.duration))?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> NewTransaction<'_> {
        NewTransaction {
            service: Service::ResourceBundle,
            params: Params {
                address: &self.address,
                amounts: Some(Amounts { energy: Some(self.energy), bandwidth: Some(self.bandwidth) }),
                duration: Some(self.duration),
                activate_address: self.activate_address,
            },
            external_id: self.external_id.as_deref(),
        }
    }
}

/// A TRON address activation. An address must be activated once before it can
/// hold resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressActivationRequest {
    address: String,
    external_id: Option<String>,
}

impl AddressActivationRequest {
    /// Activates `address`.
    pub fn new(address: impl Into<String>) -> Self {
        AddressActivationRequest { address: address.into(), external_id: None }
    }

    /// Sets your own identifier for the transaction.
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("address", &self.address)?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> NewTransaction<'_> {
        NewTransaction {
            service: Service::ActivateAddress,
            params: Params { address: &self.address, amounts: None, duration: None, activate_address: false },
            external_id: self.external_id.as_deref(),
        }
    }
}

/// The transaction to look up, by its TronZap identifier, your external
/// identifier, or both.
///
/// ```
/// use tronzap_sdk::requests::CheckTransactionRequest;
///
/// let by_id = CheckTransactionRequest::by_id("a1b2c3");
/// let by_yours = CheckTransactionRequest::by_external_id("order-42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckTransactionRequest {
    id: Option<String>,
    external_id: Option<String>,
}

impl CheckTransactionRequest {
    /// Looks a transaction up by the identifier TronZap assigned to it.
    pub fn by_id(id: impl Into<String>) -> Self {
        CheckTransactionRequest { id: Some(id.into()), external_id: None }
    }

    /// Looks a transaction up by the external identifier you set when creating it.
    pub fn by_external_id(external_id: impl Into<String>) -> Self {
        CheckTransactionRequest { id: None, external_id: Some(external_id.into()) }
    }

    /// Adds the TronZap identifier to the lookup.
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Adds your external identifier to the lookup.
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        optional("id", self.id.as_deref())?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            external_id: Option<&'a str>,
        }
        Wire { id: self.id.as_deref(), external_id: self.external_id.as_deref() }
    }
}

#[derive(Serialize)]
pub(crate) struct NewTransaction<'a> {
    service: Service,
    params: Params<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_id: Option<&'a str>,
}

#[derive(Serialize)]
struct Params<'a> {
    address: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    amounts: Option<Amounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    activate_address: bool,
}

#[derive(Serialize)]
struct Amounts {
    #[serde(skip_serializing_if = "Option::is_none")]
    energy: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bandwidth: Option<u64>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn wire(value: impl Serialize) -> Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn energy_wire_format() {
        let request = EnergyTransactionRequest::new("TAddr", 65000);
        assert_eq!(
            wire(request.wire()),
            json!({"service": "energy", "params": {"address": "TAddr", "amounts": {"energy": 65000}, "duration": 1}})
        );
        let request = request.duration(24).external_id("order-42").activate_address(true);
        assert_eq!(
            wire(request.wire()),
            json!({
                "service": "energy",
                "params": {"address": "TAddr", "amounts": {"energy": 65000}, "duration": 24, "activate_address": true},
                "external_id": "order-42"
            })
        );
    }

    #[test]
    fn bandwidth_bundle_and_activation_wire_format() {
        assert_eq!(
            wire(BandwidthTransactionRequest::new("TAddr", 345).wire()),
            json!({"service": "bandwidth", "params": {"address": "TAddr", "amounts": {"bandwidth": 345}, "duration": 1}})
        );
        assert_eq!(
            wire(ResourceBundleTransactionRequest::new("TAddr", 65000, 345).wire()),
            json!({
                "service": "resource_bundle",
                "params": {"address": "TAddr", "amounts": {"energy": 65000, "bandwidth": 345}, "duration": 1}
            })
        );
        assert_eq!(
            wire(AddressActivationRequest::new("TAddr").external_id("act-1").wire()),
            json!({"service": "activate_address", "params": {"address": "TAddr"}, "external_id": "act-1"})
        );
    }

    #[test]
    fn check_sends_only_the_given_ids() {
        assert_eq!(wire(CheckTransactionRequest::by_id("tx").wire()), json!({"id": "tx"}));
        assert_eq!(
            wire(CheckTransactionRequest::by_external_id("ext").id("tx").wire()),
            json!({"id": "tx", "external_id": "ext"})
        );
    }

    #[test]
    fn rejects_invalid_values() {
        assert_eq!(EnergyTransactionRequest::new("", 65000).validate().unwrap_err().field(), "address");
        assert_eq!(EnergyTransactionRequest::new("T", 0).validate().unwrap_err().field(), "energy");
        assert_eq!(
            EnergyTransactionRequest::new("T", 1).duration(0).validate().unwrap_err().field(),
            "duration"
        );
        assert_eq!(
            EnergyTransactionRequest::new("T", 1).external_id(" ").validate().unwrap_err().field(),
            "external_id"
        );
        assert_eq!(BandwidthTransactionRequest::new("T", 0).validate().unwrap_err().field(), "bandwidth");
        assert_eq!(
            ResourceBundleTransactionRequest::new("T", 65000, 0).validate().unwrap_err().field(),
            "bandwidth"
        );
        assert_eq!(AddressActivationRequest::new(" ").validate().unwrap_err().field(), "address");
        assert_eq!(CheckTransactionRequest::by_id("").validate().unwrap_err().field(), "id");
    }
}
