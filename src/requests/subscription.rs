use serde::Serialize;

use super::{optional, positive, required};
use crate::error::ValidationError;
use crate::models::SubscriptionStatus;

/// A subscription to start for an address.
///
/// `duration_days` and `transactions_limit` default to 0, which means no time
/// limit and no transactions limit.
///
/// ```
/// use tronzap_sdk::requests::StartSubscriptionRequest;
///
/// let request = StartSubscriptionRequest::new("unlimited_energy", "TRecipientAddress")
///     .duration_days(30)
///     .external_id("sub-42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartSubscriptionRequest {
    subscription_id: String,
    address: String,
    duration_days: u32,
    transactions_limit: u64,
    external_id: Option<String>,
    activate_address: bool,
}

impl StartSubscriptionRequest {
    /// Subscribes `address` to the plan `subscription_id`.
    ///
    /// `subscription_id` is a plan key such as `unlimited_energy`, the
    /// [`SubscriptionPlan::subscription_id`](crate::responses::SubscriptionPlan::subscription_id)
    /// that [`get_subscriptions`](crate::TronzapClient::get_subscriptions) returns, not
    /// the plan's numeric `id`.
    pub fn new(subscription_id: impl Into<String>, address: impl Into<String>) -> Self {
        StartSubscriptionRequest {
            subscription_id: subscription_id.into(),
            address: address.into(),
            duration_days: 0,
            transactions_limit: 0,
            external_id: None,
            activate_address: false,
        }
    }

    /// Sets how many days the subscription runs. Defaults to 0, no time limit.
    #[must_use]
    pub fn duration_days(mut self, days: u32) -> Self {
        self.duration_days = days;
        self
    }

    /// Sets how many transactions the subscription covers. Defaults to 0, no limit.
    #[must_use]
    pub fn transactions_limit(mut self, limit: u64) -> Self {
        self.transactions_limit = limit;
        self
    }

    /// Sets your own identifier for the subscription, to look it up later with
    /// [`SubscriptionRequest::by_external_id`].
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Also activates the address when it is not active yet.
    #[must_use]
    pub fn activate_address(mut self, activate: bool) -> Self {
        self.activate_address = activate;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("subscription_id", &self.subscription_id)?;
        required("address", &self.address)?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            subscription_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            external_id: Option<&'a str>,
            params: Params<'a>,
        }
        #[derive(Serialize)]
        struct Params<'a> {
            address: &'a str,
            duration: u32,
            transactions_limit: u64,
            #[serde(skip_serializing_if = "std::ops::Not::not")]
            activate_address: bool,
        }
        Wire {
            subscription_id: &self.subscription_id,
            external_id: self.external_id.as_deref(),
            params: Params {
                address: &self.address,
                duration: self.duration_days,
                transactions_limit: self.transactions_limit,
                activate_address: self.activate_address,
            },
        }
    }
}

/// The subscription to check or stop, by its TronZap identifier, your external
/// identifier, or both.
///
/// ```
/// use tronzap_sdk::requests::SubscriptionRequest;
///
/// let by_id = SubscriptionRequest::by_id("01m4e1z3q0r7x225zc6p63m5ey");
/// let by_yours = SubscriptionRequest::by_external_id("sub-42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionRequest {
    id: Option<String>,
    external_id: Option<String>,
}

impl SubscriptionRequest {
    /// Looks a subscription up by the identifier TronZap assigned to it.
    pub fn by_id(id: impl Into<String>) -> Self {
        SubscriptionRequest { id: Some(id.into()), external_id: None }
    }

    /// Looks a subscription up by the external identifier you set when starting it.
    pub fn by_external_id(external_id: impl Into<String>) -> Self {
        SubscriptionRequest { id: None, external_id: Some(external_id.into()) }
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

/// A page of subscription history.
///
/// ```
/// use tronzap_sdk::models::SubscriptionStatus;
/// use tronzap_sdk::requests::SubscriptionHistoryRequest;
///
/// let first_page = SubscriptionHistoryRequest::new();
/// let active = SubscriptionHistoryRequest::new().page(2).per_page(50).status(SubscriptionStatus::Active);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionHistoryRequest {
    page: u32,
    per_page: u32,
    status: Option<SubscriptionStatus>,
}

impl SubscriptionHistoryRequest {
    /// The page size used unless [`SubscriptionHistoryRequest::per_page`] is called.
    pub const DEFAULT_PER_PAGE: u32 = 10;

    /// Returns the first page of all subscriptions, 10 per page.
    pub fn new() -> Self {
        SubscriptionHistoryRequest { page: 1, per_page: Self::DEFAULT_PER_PAGE, status: None }
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

    /// Returns only subscriptions in this state.
    #[must_use]
    pub fn status(mut self, status: SubscriptionStatus) -> Self {
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
            status: Option<&'a SubscriptionStatus>,
        }
        Wire { page: self.page, per_page: self.per_page, status: self.status.as_ref() }
    }
}

impl Default for SubscriptionHistoryRequest {
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
    fn start_sends_zero_limits() {
        assert_eq!(
            wire(StartSubscriptionRequest::new("unlimited_energy", "TAddress").wire()),
            json!({
                "subscription_id": "unlimited_energy",
                "params": {"address": "TAddress", "duration": 0, "transactions_limit": 0}
            })
        );
    }

    #[test]
    fn start_wire_format() {
        let request = StartSubscriptionRequest::new("unlimited_energy", "TAddress")
            .duration_days(30)
            .transactions_limit(10)
            .external_id("0")
            .activate_address(true);
        assert_eq!(
            serde_json::to_string(&request.wire()).unwrap(),
            r#"{"subscription_id":"unlimited_energy","external_id":"0","params":{"address":"TAddress","duration":30,"transactions_limit":10,"activate_address":true}}"#
        );
    }

    #[test]
    fn lookup_sends_only_the_given_ids() {
        assert_eq!(wire(SubscriptionRequest::by_id("sub-1").wire()), json!({"id": "sub-1"}));
        assert_eq!(
            wire(SubscriptionRequest::by_external_id("ext-1").wire()),
            json!({"external_id": "ext-1"})
        );
        assert_eq!(
            wire(SubscriptionRequest::by_external_id("ext-1").id("sub-1").wire()),
            json!({"id": "sub-1", "external_id": "ext-1"})
        );
    }

    #[test]
    fn history_defaults() {
        assert_eq!(wire(SubscriptionHistoryRequest::default().wire()), json!({"page": 1, "per_page": 10}));
        assert_eq!(
            wire(
                SubscriptionHistoryRequest::new()
                    .page(2)
                    .per_page(50)
                    .status(SubscriptionStatus::Active)
                    .wire()
            ),
            json!({"page": 2, "per_page": 50, "status": "active"})
        );
    }

    #[test]
    fn rejects_invalid_values() {
        let field = |r: StartSubscriptionRequest| r.validate().unwrap_err().field().to_owned();
        assert_eq!(field(StartSubscriptionRequest::new("", "TAddress")), "subscription_id");
        assert_eq!(field(StartSubscriptionRequest::new("unlimited_energy", " ")), "address");
        assert_eq!(
            field(StartSubscriptionRequest::new("unlimited_energy", "TAddress").external_id("")),
            "external_id"
        );
        assert_eq!(SubscriptionRequest::by_id("").validate().unwrap_err().field(), "id");
        assert_eq!(SubscriptionRequest::by_external_id(" ").validate().unwrap_err().field(), "external_id");
        assert_eq!(SubscriptionHistoryRequest::new().page(0).validate().unwrap_err().field(), "page");
        assert_eq!(SubscriptionHistoryRequest::new().per_page(0).validate().unwrap_err().field(), "per_page");
    }
}
