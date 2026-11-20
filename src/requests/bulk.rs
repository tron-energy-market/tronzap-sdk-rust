use serde::Serialize;

use super::{DEFAULT_DURATION, optional, positive, required};
use crate::error::ValidationError;
use crate::models::BulkRowStatus;

/// One row of a bulk energy order.
///
/// The SDK does not check the address or the amount: the calculation rejects
/// invalid rows and reports them with their row number in
/// [`BulkCalculation::rejected`](crate::responses::BulkCalculation::rejected).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BulkItem {
    address: String,
    amount: u64,
}

impl BulkItem {
    /// Orders `amount` units of energy for `address`.
    pub fn new(address: impl Into<String>, amount: u64) -> Self {
        BulkItem { address: address.into(), amount }
    }

    /// Returns the address that receives the energy.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the amount of energy.
    pub fn amount(&self) -> u64 {
        self.amount
    }
}

/// The rows of a bulk energy order to validate and price.
///
/// ```
/// use tronzap_sdk::requests::{BulkCalculateRequest, BulkItem};
///
/// let request = BulkCalculateRequest::new([
///     BulkItem::new("TRecipientAddress1", 65000),
///     BulkItem::new("TRecipientAddress2", 131000),
/// ])
/// .duration(1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkCalculateRequest {
    items: Vec<BulkItem>,
    duration: u32,
}

impl BulkCalculateRequest {
    /// Prices the rows, at most 10 000, rented for 1 hour.
    pub fn new(items: impl IntoIterator<Item = BulkItem>) -> Self {
        BulkCalculateRequest { items: items.into_iter().collect(), duration: DEFAULT_DURATION }
    }

    /// Sets the rental duration in hours, one of the durations
    /// [`get_services`](crate::TronzapClient::get_services) lists. Defaults to 1.
    #[must_use]
    pub fn duration(mut self, hours: u32) -> Self {
        self.duration = hours;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        if self.items.is_empty() {
            return Err(ValidationError::new("items", "is required"));
        }
        positive("duration", u64::from(self.duration))
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            resource: &'static str,
            duration: u32,
            items: &'a [BulkItem],
        }
        Wire { resource: "energy", duration: self.duration, items: &self.items }
    }
}

/// A bulk order to start from a calculation.
///
/// ```
/// use tronzap_sdk::requests::BulkCreateRequest;
///
/// let request = BulkCreateRequest::new("01kb2m7a1bc2de3fg4hj5km6np")
///     .external_id("order-42")
///     .webhook_url("https://example.com/tronzap/bulk");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkCreateRequest {
    id: String,
    external_id: Option<String>,
    force: bool,
    max_price_increase: Option<u32>,
    webhook_url: Option<String>,
}

impl BulkCreateRequest {
    /// Starts the calculation with the
    /// [`BulkCalculation::id`](crate::responses::BulkCalculation::id) `id`.
    pub fn new(id: impl Into<String>) -> Self {
        BulkCreateRequest {
            id: id.into(),
            external_id: None,
            force: false,
            max_price_increase: None,
            webhook_url: None,
        }
    }

    /// Sets your own identifier for the order, unique per account. Starting an
    /// order again with the same external identifier returns the order already
    /// started.
    #[must_use]
    pub fn external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Buys even if the price rises while the order runs, within
    /// [`max_price_increase`](Self::max_price_increase). Without it, an address
    /// whose price rose is skipped.
    #[must_use]
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Sets the allowed price increase in percent, 0 to 100, used only with
    /// [`force`](Self::force). The API default is 20.
    #[must_use]
    pub fn max_price_increase(mut self, percent: u32) -> Self {
        self.max_price_increase = Some(percent);
        self
    }

    /// Sets a public `https` URL the API calls once when the order becomes final.
    /// Check its `X-Signature` header with
    /// [`verify_webhook_signature`](crate::TronzapClient::verify_webhook_signature).
    #[must_use]
    pub fn webhook_url(mut self, url: impl Into<String>) -> Self {
        self.webhook_url = Some(url.into());
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        required("id", &self.id)?;
        optional("external_id", self.external_id.as_deref())?;
        optional("webhook_url", self.webhook_url.as_deref())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire<'a> {
            id: &'a str,
            #[serde(skip_serializing_if = "std::ops::Not::not")]
            force: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_price_increase: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            webhook_url: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            external_id: Option<&'a str>,
        }
        Wire {
            id: &self.id,
            force: self.force,
            max_price_increase: self.max_price_increase,
            webhook_url: self.webhook_url.as_deref(),
            external_id: self.external_id.as_deref(),
        }
    }
}

/// The bulk order to check, by its TronZap identifier, your external identifier,
/// or both.
///
/// ```
/// use tronzap_sdk::requests::BulkOrderRequest;
///
/// let by_id = BulkOrderRequest::by_id("01kb2m5x9qj4v8n3c7d6e5f4g3");
/// let by_yours = BulkOrderRequest::by_external_id("order-42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkOrderRequest {
    id: Option<String>,
    external_id: Option<String>,
}

impl BulkOrderRequest {
    /// Looks a bulk order up by the identifier TronZap assigned to it.
    pub fn by_id(id: impl Into<String>) -> Self {
        BulkOrderRequest { id: Some(id.into()), external_id: None }
    }

    /// Looks a bulk order up by the external identifier you set when starting it.
    pub fn by_external_id(external_id: impl Into<String>) -> Self {
        BulkOrderRequest { id: None, external_id: Some(external_id.into()) }
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        optional("id", self.id.as_deref())?;
        optional("external_id", self.external_id.as_deref())
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        ids(self.id.as_deref(), self.external_id.as_deref(), None)
    }
}

/// The bulk order whose report to download, by its TronZap identifier or your
/// external identifier.
///
/// ```
/// use tronzap_sdk::models::BulkRowStatus;
/// use tronzap_sdk::requests::BulkReportRequest;
///
/// let all_rows = BulkReportRequest::by_id("01kb2m5x9qj4v8n3c7d6e5f4g3");
/// let skipped = BulkReportRequest::by_external_id("order-42").status(BulkRowStatus::Skipped);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkReportRequest {
    id: Option<String>,
    external_id: Option<String>,
    status: Option<BulkRowStatus>,
}

impl BulkReportRequest {
    /// Downloads the report of a bulk order by the identifier TronZap assigned to it.
    pub fn by_id(id: impl Into<String>) -> Self {
        BulkReportRequest { id: Some(id.into()), external_id: None, status: None }
    }

    /// Downloads the report of a bulk order by the external identifier you set
    /// when starting it.
    pub fn by_external_id(external_id: impl Into<String>) -> Self {
        BulkReportRequest { id: None, external_id: Some(external_id.into()), status: None }
    }

    /// Returns only the rows with this result.
    #[must_use]
    pub fn status(mut self, status: BulkRowStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        optional("id", self.id.as_deref())?;
        optional("external_id", self.external_id.as_deref())?;
        match &self.status {
            Some(BulkRowStatus::Unknown(_)) => {
                Err(ValidationError::new("status", "is not a known row status"))
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        ids(self.id.as_deref(), self.external_id.as_deref(), self.status.as_ref())
    }
}

/// A page of bulk orders.
///
/// ```
/// use tronzap_sdk::requests::BulkHistoryRequest;
///
/// let second_page = BulkHistoryRequest::new().page(2).per_page(50);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkHistoryRequest {
    page: u32,
    per_page: u32,
}

impl BulkHistoryRequest {
    /// The page size used unless [`BulkHistoryRequest::per_page`] is called.
    pub const DEFAULT_PER_PAGE: u32 = 10;

    /// Returns the first page, 10 orders per page.
    pub fn new() -> Self {
        BulkHistoryRequest { page: 1, per_page: Self::DEFAULT_PER_PAGE }
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

    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        positive("page", u64::from(self.page))?;
        positive("per_page", u64::from(self.per_page))
    }

    pub(crate) fn wire(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct Wire {
            page: u32,
            per_page: u32,
        }
        Wire { page: self.page, per_page: self.per_page }
    }
}

impl Default for BulkHistoryRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct Ids<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a BulkRowStatus>,
}

fn ids<'a>(id: Option<&'a str>, external_id: Option<&'a str>, status: Option<&'a BulkRowStatus>) -> Ids<'a> {
    Ids { id, external_id, status }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn wire(value: impl Serialize) -> Value {
        serde_json::to_value(value).unwrap()
    }

    #[test]
    fn calculate_sends_energy_and_the_rows() {
        let request = BulkCalculateRequest::new([BulkItem::new("TAddr1", 65000), BulkItem::new("abc", 1)]);
        assert_eq!(
            wire(request.wire()),
            json!({"resource": "energy", "duration": 1, "items": [
                {"address": "TAddr1", "amount": 65000}, {"address": "abc", "amount": 1}
            ]})
        );
        assert_eq!(wire(request.duration(24).wire())["duration"], 24);
    }

    #[test]
    fn create_sends_only_what_is_set() {
        assert_eq!(wire(BulkCreateRequest::new("calc-1").wire()), json!({"id": "calc-1"}));
        let request = BulkCreateRequest::new("calc-1")
            .external_id("order-42")
            .force(true)
            .max_price_increase(0)
            .webhook_url("https://example.com/hook");
        assert_eq!(
            serde_json::to_string(&request.wire()).unwrap(),
            r#"{"id":"calc-1","force":true,"max_price_increase":0,"webhook_url":"https://example.com/hook","external_id":"order-42"}"#
        );
    }

    #[test]
    fn lookups_send_the_ids_and_the_status() {
        assert_eq!(wire(BulkOrderRequest::by_id("bulk-1").wire()), json!({"id": "bulk-1"}));
        assert_eq!(
            wire(BulkOrderRequest::by_external_id("order-42").wire()),
            json!({"external_id": "order-42"})
        );
        assert_eq!(
            wire(BulkReportRequest::by_id("bulk-1").status(BulkRowStatus::InReview).wire()),
            json!({"id": "bulk-1", "status": "in_review"})
        );
        assert_eq!(wire(BulkHistoryRequest::default().wire()), json!({"page": 1, "per_page": 10}));
    }

    #[test]
    fn rejects_invalid_values() {
        assert_eq!(BulkCalculateRequest::new([]).validate().unwrap_err().field(), "items");
        assert_eq!(
            BulkCalculateRequest::new([BulkItem::new("T", 1)]).duration(0).validate().unwrap_err().field(),
            "duration"
        );
        assert_eq!(BulkCreateRequest::new(" ").validate().unwrap_err().field(), "id");
        assert_eq!(
            BulkCreateRequest::new("c").webhook_url("").validate().unwrap_err().field(),
            "webhook_url"
        );
        assert_eq!(BulkOrderRequest::by_id("").validate().unwrap_err().field(), "id");
        assert_eq!(
            BulkReportRequest::by_id("b").status(BulkRowStatus::from("lost")).validate().unwrap_err().field(),
            "status"
        );
        assert_eq!(BulkHistoryRequest::new().per_page(0).validate().unwrap_err().field(), "per_page");
    }
}
