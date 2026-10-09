//! Results of the API calls.
//!
//! Collections are never missing: an absent list decodes as empty. Values the API
//! may omit are `Option`s. All structs are `#[non_exhaustive]`, so fields can be
//! added in minor releases.

mod de;

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::models::{
    AmlCheckType, AmlDirection, AmlRiskLevel, AmlStatus, Decimal, Service, SubscriptionStatus, Timestamp,
    TransactionStatus,
};

/// The resources on sale and their prices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Services {
    /// One price tier per energy amount range and duration.
    #[serde(default, deserialize_with = "de::list")]
    pub energy: Vec<EnergyRate>,
    /// One price tier per bandwidth amount range and duration.
    #[serde(default, deserialize_with = "de::list")]
    pub bandwidth: Vec<BandwidthRate>,
    /// The address activation fee, when the API lists one.
    #[serde(default, deserialize_with = "de::opt_object")]
    pub activate_address: Option<ActivateAddressRate>,
}

/// One energy price tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawEnergyRate")]
#[non_exhaustive]
pub struct EnergyRate {
    /// The rental duration in hours.
    pub duration: u32,
    /// The smallest amount in this tier.
    pub min_amount: u64,
    /// The largest amount in this tier.
    pub max_amount: u64,
    /// The smallest energy amount in this tier. Always equal to `min_amount`.
    #[deprecated(since = "1.1.0", note = "use `min_amount`")]
    pub min_energy: u64,
    /// The largest energy amount in this tier. Always equal to `max_amount`.
    #[deprecated(since = "1.1.0", note = "use `max_amount`")]
    pub max_energy: u64,
    /// The price of **1000 units** of energy: 65000 energy at a price of 0.03
    /// costs 1.95. Bandwidth is priced the same way, see [`BandwidthRate::price`].
    #[serde(with = "de::decimal")]
    pub price: Decimal,
    /// The price of 32,000 energy.
    #[serde(with = "de::decimal")]
    pub price_32k: Decimal,
    /// The price of 65,000 energy.
    #[serde(with = "de::decimal")]
    pub price_65k: Decimal,
    /// The price of 131,000 energy.
    #[serde(with = "de::decimal")]
    pub price_131k: Decimal,
}

#[derive(Deserialize)]
struct RawEnergyRate {
    #[serde(default, deserialize_with = "de::u32")]
    duration: u32,
    #[serde(default, deserialize_with = "de::u64")]
    min_amount: u64,
    #[serde(default, deserialize_with = "de::u64")]
    max_amount: u64,
    #[serde(default, with = "de::decimal")]
    price: Decimal,
    #[serde(default, with = "de::decimal")]
    price_32k: Decimal,
    #[serde(default, with = "de::decimal")]
    price_65k: Decimal,
    #[serde(default, with = "de::decimal")]
    price_131k: Decimal,
}

impl From<RawEnergyRate> for EnergyRate {
    #[allow(deprecated)]
    fn from(raw: RawEnergyRate) -> Self {
        EnergyRate {
            duration: raw.duration,
            min_amount: raw.min_amount,
            max_amount: raw.max_amount,
            min_energy: raw.min_amount,
            max_energy: raw.max_amount,
            price: raw.price,
            price_32k: raw.price_32k,
            price_65k: raw.price_65k,
            price_131k: raw.price_131k,
        }
    }
}

/// One bandwidth price tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BandwidthRate {
    /// The rental duration in hours.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration: u32,
    /// The smallest amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub min_amount: u64,
    /// The largest amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub max_amount: u64,
    /// The price of **1000 units** of bandwidth: 345 bandwidth at a price of 1
    /// costs 0.345. Energy is priced the same way, see [`EnergyRate::price`].
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
}

/// The address activation fee.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActivateAddressRate {
    /// The flat fee.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
}

/// An AML screening product and its price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AmlService {
    /// The service identifier.
    #[serde(default, deserialize_with = "de::string")]
    pub id: String,
    /// What the service screens.
    #[serde(rename = "type", default = "de::unknown", deserialize_with = "de::wire_enum")]
    pub check_type: AmlCheckType,
    /// The price of one check.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
}

#[derive(Deserialize)]
#[serde(transparent)]
pub(crate) struct AmlServices(#[serde(deserialize_with = "de::list")] pub(crate) Vec<AmlService>);

/// The account balance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Balance {
    /// The available balance.
    #[serde(default, with = "de::decimal")]
    pub balance: Decimal,
    /// The TRON address that tops the account up.
    #[serde(default, deserialize_with = "de::string")]
    pub address: String,
}

/// The on-chain resources and token balances of an address.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AddressInfo {
    /// The available energy and bandwidth.
    #[serde(default, deserialize_with = "de::object")]
    pub resources: Resources,
    /// Token balances by symbol, such as `TRX` and `USDT`.
    #[serde(default, with = "de::decimal_map")]
    pub balances: BTreeMap<String, Decimal>,
}

/// The available amount of each TRON resource.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Resources {
    /// The available energy.
    #[serde(default, deserialize_with = "de::u64")]
    pub energy: u64,
    /// The available bandwidth.
    #[serde(default, deserialize_with = "de::u64")]
    pub bandwidth: u64,
}

/// The energy a transfer needs, and its cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawEnergyEstimate")]
#[non_exhaustive]
pub struct EnergyEstimate {
    /// The estimated energy. Buy this much to cover the transfer.
    pub amount: u64,
    /// The estimated energy. Always equal to `amount`.
    #[deprecated(since = "1.1.0", note = "use `amount`")]
    pub energy: u64,
    /// The rental duration in hours the price refers to.
    pub duration: u32,
    /// The cost of the energy.
    #[serde(with = "de::decimal")]
    pub price: Decimal,
    /// The address activation fee included in `total`, if any.
    #[serde(with = "de::decimal")]
    pub activation_fee: Decimal,
    /// The total cost.
    #[serde(with = "de::decimal")]
    pub total: Decimal,
    /// The sender address.
    pub from_address: String,
    /// The recipient address.
    pub to_address: String,
    /// The token contract the estimate refers to.
    pub contract_address: String,
}

#[derive(Deserialize)]
struct RawEnergyEstimate {
    #[serde(default, deserialize_with = "de::u64")]
    amount: u64,
    #[serde(default, deserialize_with = "de::u32")]
    duration: u32,
    #[serde(default, with = "de::decimal")]
    price: Decimal,
    #[serde(default, with = "de::decimal")]
    activation_fee: Decimal,
    #[serde(default, with = "de::decimal")]
    total: Decimal,
    #[serde(default, deserialize_with = "de::string")]
    from_address: String,
    #[serde(default, deserialize_with = "de::string")]
    to_address: String,
    #[serde(default, deserialize_with = "de::string")]
    contract_address: String,
}

impl From<RawEnergyEstimate> for EnergyEstimate {
    #[allow(deprecated)]
    fn from(raw: RawEnergyEstimate) -> Self {
        EnergyEstimate {
            amount: raw.amount,
            energy: raw.amount,
            duration: raw.duration,
            price: raw.price,
            activation_fee: raw.activation_fee,
            total: raw.total,
            from_address: raw.from_address,
            to_address: raw.to_address,
            contract_address: raw.contract_address,
        }
    }
}

/// The price of a purchase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawCalculation")]
#[non_exhaustive]
pub struct Calculation {
    /// The address the price was calculated for.
    pub address: String,
    /// The priced service.
    #[serde(rename = "type")]
    pub service: Service,
    /// The priced resource amount.
    pub amount: u64,
    /// The priced energy amount. Always equal to `amount`.
    #[deprecated(since = "1.1.0", note = "use `amount`")]
    pub energy: u64,
    /// The rental duration in hours.
    pub duration: u32,
    /// The cost of the resources.
    #[serde(with = "de::decimal")]
    pub price: Decimal,
    /// The address activation fee included in `total`, if any.
    #[serde(with = "de::decimal")]
    pub activation_fee: Decimal,
    /// The total cost.
    #[serde(with = "de::decimal")]
    pub total: Decimal,
}

#[derive(Deserialize)]
struct RawCalculation {
    #[serde(default, deserialize_with = "de::string")]
    address: String,
    #[serde(rename = "type", default = "de::unknown", deserialize_with = "de::wire_enum")]
    service: Service,
    #[serde(default, deserialize_with = "de::u64")]
    amount: u64,
    #[serde(default, deserialize_with = "de::u32")]
    duration: u32,
    #[serde(default, with = "de::decimal")]
    price: Decimal,
    #[serde(default, with = "de::decimal")]
    activation_fee: Decimal,
    #[serde(default, with = "de::decimal")]
    total: Decimal,
}

impl From<RawCalculation> for Calculation {
    #[allow(deprecated)]
    fn from(raw: RawCalculation) -> Self {
        Calculation {
            address: raw.address,
            service: raw.service,
            amount: raw.amount,
            energy: raw.amount,
            duration: raw.duration,
            price: raw.price,
            activation_fee: raw.activation_fee,
            total: raw.total,
        }
    }
}

/// A resource purchase or an address activation.
///
/// The API currently reports a resource bundle with `service` equal to
/// [`Service::Energy`], not [`Service::ResourceBundle`]. Read
/// [`TransactionParams::amounts`] to see which resources a transaction holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawTransaction")]
#[non_exhaustive]
pub struct Transaction {
    /// The identifier TronZap assigned.
    pub id: String,
    /// The external identifier you set, if any.
    pub external_id: Option<String>,
    /// The purchased service.
    pub service: Service,
    /// The parameters the transaction was created with.
    pub params: TransactionParams,
    /// The current state.
    pub status: TransactionStatus,
    /// The amount charged to the balance.
    #[serde(with = "de::decimal")]
    pub amount: Decimal,
    /// When the transaction was created.
    pub created_at: Option<Timestamp>,
    /// The on-chain transaction hash, once the transaction is settled.
    pub hash: Option<String>,
}

/// The parameters a transaction was created with.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TransactionParams {
    /// The address that receives the resources.
    pub address: String,
    /// The rental duration in hours.
    pub duration: u32,
    /// The purchased amount of each resource.
    pub amounts: ResourceAmounts,
    /// Whether address activation was requested.
    pub activate_address: bool,
}

/// The amount of each resource in a purchase.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResourceAmounts {
    /// The energy amount.
    #[serde(default, deserialize_with = "de::u64")]
    pub energy: u64,
    /// The bandwidth amount.
    #[serde(default, deserialize_with = "de::u64")]
    pub bandwidth: u64,
}

#[derive(Deserialize)]
struct RawTransaction {
    #[serde(default, deserialize_with = "de::string")]
    id: String,
    #[serde(default, deserialize_with = "de::opt_string")]
    external_id: Option<String>,
    #[serde(default = "de::unknown", deserialize_with = "de::wire_enum")]
    service: Service,
    #[serde(default, deserialize_with = "de::object")]
    params: RawParams,
    #[serde(default = "de::unknown", deserialize_with = "de::wire_enum")]
    status: TransactionStatus,
    #[serde(default, with = "de::decimal")]
    amount: Decimal,
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    created_at: Option<Timestamp>,
    #[serde(default, deserialize_with = "de::opt_string")]
    hash: Option<String>,
}

#[derive(Default, Deserialize)]
struct RawParams {
    #[serde(default, deserialize_with = "de::string")]
    address: String,
    #[serde(default, deserialize_with = "de::u32")]
    duration: u32,
    #[serde(default, deserialize_with = "de::object")]
    amounts: ResourceAmounts,
    #[serde(default, deserialize_with = "de::u64")]
    amount: u64,
    #[serde(default, deserialize_with = "de::u64")]
    energy_amount: u64,
    #[serde(default, deserialize_with = "de::bool")]
    activate_address: bool,
}

impl From<RawTransaction> for Transaction {
    fn from(raw: RawTransaction) -> Self {
        let p = raw.params;
        let mut amounts = p.amounts;
        // Older transactions carry params.amount / params.energy_amount instead of params.amounts.
        if amounts.energy == 0 && amounts.bandwidth == 0 {
            amounts.energy = p.energy_amount;
            if raw.service == Service::Bandwidth {
                amounts.bandwidth = p.amount;
            } else if amounts.energy == 0 {
                amounts.energy = p.amount;
            }
        }
        Transaction {
            id: raw.id,
            external_id: raw.external_id,
            service: raw.service,
            params: TransactionParams {
                address: p.address,
                duration: p.duration,
                amounts,
                activate_address: p.activate_address,
            },
            status: raw.status,
            amount: raw.amount,
            created_at: raw.created_at,
            hash: raw.hash,
        }
    }
}

/// The direct recharge service: pay the returned address and energy is delivered
/// at the listed rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DirectRechargeInfo {
    /// The TronZap address to send payment to.
    #[serde(default, deserialize_with = "de::string")]
    pub address: String,
    /// The available rates.
    #[serde(default, deserialize_with = "de::list")]
    pub rates: Vec<DirectRechargeRate>,
}

/// One direct recharge price tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DirectRechargeRate {
    /// The rental duration in hours.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration: u32,
    /// The smallest energy amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub min_energy: u64,
    /// The largest energy amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub max_energy: u64,
    /// The price of 1000 units of energy.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
    /// The price of 32,000 energy.
    #[serde(default, with = "de::decimal")]
    pub price_32k: Decimal,
    /// The price of 65,000 energy.
    #[serde(default, with = "de::decimal")]
    pub price_65k: Decimal,
    /// The price of 131,000 energy.
    #[serde(default, with = "de::decimal")]
    pub price_131k: Decimal,
}

/// An AML screening and, once completed, its result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AmlCheck {
    /// The check identifier.
    #[serde(default, deserialize_with = "de::string")]
    pub id: String,
    /// What is screened.
    #[serde(rename = "type", default = "de::unknown", deserialize_with = "de::wire_enum")]
    pub check_type: AmlCheckType,
    /// The screened address. For a hash check, the recipient address of the
    /// transaction.
    #[serde(default, deserialize_with = "de::string")]
    pub address: String,
    /// The screened transaction hash, for a hash check.
    #[serde(default, deserialize_with = "de::opt_string")]
    pub hash: Option<String>,
    /// Which side of the screened transaction you are on, for a hash check.
    #[serde(default, deserialize_with = "de::opt_wire_enum")]
    pub direction: Option<AmlDirection>,
    /// The network code.
    #[serde(default, deserialize_with = "de::string")]
    pub network: String,
    /// The current state.
    #[serde(default = "de::unknown", deserialize_with = "de::wire_enum")]
    pub status: AmlStatus,
    /// The risk score from 0 to 100. `None` until the check completes; a completed
    /// check can score 0, which is not the same as having no score yet.
    #[serde(default, with = "de::opt_decimal")]
    pub risk_score: Option<Decimal>,
    /// The risk level, once the check completes.
    #[serde(default, deserialize_with = "de::opt_wire_enum")]
    pub risk_level: Option<AmlRiskLevel>,
    /// Whether the subject is on a blacklist.
    #[serde(default, deserialize_with = "de::bool")]
    pub blacklist: bool,
    /// The signals that contributed to the score.
    #[serde(default, deserialize_with = "de::list")]
    pub risk_factors: Vec<AmlRiskFactor>,
    /// When the screening ran.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub checked_at: Option<Timestamp>,
}

/// One signal that contributed to an AML risk score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AmlRiskFactor {
    /// The machine-readable name.
    #[serde(default, deserialize_with = "de::string")]
    pub name: String,
    /// The human-readable name.
    #[serde(default, deserialize_with = "de::string")]
    pub label: String,
    /// The risk group, such as `low` or `medium`.
    #[serde(default, deserialize_with = "de::string")]
    pub group: String,
    /// The weight of the factor, from 0 to 1.
    #[serde(default, with = "de::decimal")]
    pub score: Decimal,
}

/// One page of AML check history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AmlHistory {
    /// The page number, starting at 1.
    #[serde(default, deserialize_with = "de::u32")]
    pub page: u32,
    /// The page size.
    #[serde(default, deserialize_with = "de::u32")]
    pub per_page: u32,
    /// The number of matching checks across all pages.
    #[serde(default, deserialize_with = "de::u64")]
    pub total: u64,
    /// The checks on this page.
    #[serde(default, deserialize_with = "de::list")]
    pub items: Vec<AmlCheck>,
}

/// A subscription plan on sale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubscriptionPlan {
    /// The plan key, such as `unlimited_energy`. Pass it to
    /// [`StartSubscriptionRequest::new`](crate::requests::StartSubscriptionRequest::new).
    #[serde(default, deserialize_with = "de::string")]
    pub subscription_id: String,
    /// The plan's numeric identifier.
    #[serde(default, deserialize_with = "de::u64")]
    pub id: u64,
    /// The human-readable plan name.
    #[serde(default, deserialize_with = "de::string")]
    pub name: String,
    /// The one-time fee charged when a subscription starts.
    #[serde(default, with = "de::decimal")]
    pub activation_fee: Decimal,
    /// The amount charged when a subscription starts.
    #[serde(default, with = "de::decimal")]
    pub initial_price: Decimal,
    /// The cost of each transaction a subscription serves.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
    /// How many transactions the plan covers, 0 for no limit.
    #[serde(default, deserialize_with = "de::u64")]
    pub transactions_limit: u64,
    /// How many days the plan runs, 0 for no time limit.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration_days: u32,
}

/// The plans object, keyed by plan, decoded into a list in the API's order.
pub(crate) struct SubscriptionPlans(pub(crate) Vec<SubscriptionPlan>);

impl<'de> Deserialize<'de> for SubscriptionPlans {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Plans;

        impl<'de> Visitor<'de> for Plans {
            type Value = SubscriptionPlans;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an object of subscription plans")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut plans = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((key, mut plan)) = map.next_entry::<String, SubscriptionPlan>()? {
                    plan.subscription_id = key;
                    plans.push(plan);
                }
                Ok(SubscriptionPlans(plans))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut plans = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(plan) = seq.next_element()? {
                    plans.push(plan);
                }
                Ok(SubscriptionPlans(plans))
            }
        }

        deserializer.deserialize_any(Plans)
    }
}

/// An energy subscription for an address.
///
/// [`start_subscription`](crate::TronzapClient::start_subscription),
/// [`check_subscription`](crate::TronzapClient::check_subscription) and
/// [`stop_subscription`](crate::TronzapClient::stop_subscription) report the
/// identifiers, status, dates and [`params`](Self::params).
/// [`get_subscription_history`](crate::TronzapClient::get_subscription_history)
/// reports the usage counters and `total_price` instead of `params`. Counters and
/// prices a response does not carry are 0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Subscription {
    /// The identifier TronZap assigned.
    #[serde(default, deserialize_with = "de::string")]
    pub id: String,
    /// The plan key, such as `unlimited_energy`.
    #[serde(default, deserialize_with = "de::string")]
    pub subscription_id: String,
    /// The external identifier you set, if any.
    #[serde(default, deserialize_with = "de::opt_string")]
    pub external_id: Option<String>,
    /// The address the subscription serves. A stopped subscription is reported
    /// without it; read [`SubscriptionParams::address`] instead.
    #[serde(default, deserialize_with = "de::opt_string")]
    pub address: Option<String>,
    /// The current state.
    #[serde(default = "de::unknown", deserialize_with = "de::wire_enum")]
    pub status: SubscriptionStatus,
    /// The parameters the subscription was started with. `None` in the history.
    #[serde(default, deserialize_with = "de::opt_object")]
    pub params: Option<SubscriptionParams>,
    /// How many transactions the subscription covers, 0 for no limit.
    #[serde(default, deserialize_with = "de::u64")]
    pub transactions_limit: u64,
    /// How many transactions the subscription has served.
    #[serde(default, deserialize_with = "de::u64")]
    pub transactions_used: u64,
    /// How much energy the subscription has delegated.
    #[serde(default, deserialize_with = "de::u64")]
    pub energy_used: u64,
    /// The amount charged for the subscription so far.
    #[serde(default, with = "de::decimal")]
    pub total_price: Decimal,
    /// When the subscription was created.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub created_at: Option<Timestamp>,
    /// When the subscription started.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub started_at: Option<Timestamp>,
    /// When the subscription was last renewed.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub renewed_at: Option<Timestamp>,
    /// When the subscription was stopped.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub stopped_at: Option<Timestamp>,
    /// When the subscription ends. `None` without a time limit, and for a
    /// stopped subscription.
    #[serde(default, deserialize_with = "de::opt_timestamp")]
    pub expire_at: Option<Timestamp>,
}

/// The parameters a subscription was started with.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubscriptionParams {
    /// The address the subscription serves.
    #[serde(default, deserialize_with = "de::string")]
    pub address: String,
    /// How many days the subscription runs, 0 for no time limit.
    #[serde(rename = "duration", default, deserialize_with = "de::u32")]
    pub duration_days: u32,
    /// How many transactions the subscription covers, 0 for no limit.
    #[serde(default, deserialize_with = "de::u64")]
    pub transactions_limit: u64,
    /// Whether address activation was requested.
    #[serde(default, deserialize_with = "de::bool")]
    pub activate_address: bool,
}

/// One page of subscription history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SubscriptionHistory {
    /// The page number, starting at 1.
    #[serde(default, deserialize_with = "de::u32")]
    pub page: u32,
    /// The page size.
    #[serde(default, deserialize_with = "de::u32")]
    pub per_page: u32,
    /// The number of matching subscriptions across all pages.
    #[serde(default, deserialize_with = "de::u64")]
    pub total: u64,
    /// The subscriptions on this page, newest first.
    #[serde(default, deserialize_with = "de::list")]
    pub items: Vec<Subscription>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> T {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn transaction_amount_as_number_and_string() {
        let a: Transaction =
            decode(json!({"id": "1", "amount": 5.47, "service": "energy", "status": "success"}));
        let b: Transaction =
            decode(json!({"id": "1", "amount": "5.47", "service": "energy", "status": "success"}));
        assert_eq!(a, b);
        assert_eq!(a.amount.to_string(), "5.47");
        assert_eq!(a.status, TransactionStatus::Success);
    }

    #[test]
    fn transaction_reads_legacy_amount_fields() {
        let t: Transaction = decode(json!({"service": "bandwidth", "params": {"amount": 345}}));
        assert_eq!(t.params.amounts, ResourceAmounts { energy: 0, bandwidth: 345 });
        let t: Transaction = decode(json!({"service": "energy", "params": {"energy_amount": 65000}}));
        assert_eq!(t.params.amounts.energy, 65000);
        let t: Transaction = decode(json!({"service": "energy", "params": {"amount": "65000"}}));
        assert_eq!(t.params.amounts.energy, 65000);
        let t: Transaction = decode(json!({"service": "energy", "params": []}));
        assert_eq!(t.params, TransactionParams::default());
    }

    #[test]
    fn transaction_with_missing_fields() {
        let t: Transaction = decode(json!({}));
        assert_eq!(t.status, TransactionStatus::Unknown(String::new()));
        assert_eq!(t.external_id, None);
        assert_eq!(t.created_at, None);
        assert!(t.amount.is_zero());
        let t: Transaction = decode(json!({"external_id": "", "hash": null, "created_at": ""}));
        assert_eq!((t.external_id, t.hash, t.created_at), (None, None, None));
    }

    #[test]
    fn risk_score_null_differs_from_zero() {
        let pending: AmlCheck = decode(json!({"status": "pending", "risk_score": null}));
        let clean: AmlCheck = decode(json!({"status": "completed", "risk_score": "0"}));
        assert_eq!(pending.risk_score, None);
        assert_eq!(clean.risk_score.map(|n| n.is_zero()), Some(true));
        let scored: AmlCheck = decode(json!({"risk_score": "35.3", "risk_level": "medium", "blacklist": 1}));
        assert_eq!(scored.risk_score.unwrap().to_string(), "35.3");
        assert_eq!(scored.risk_level, Some(AmlRiskLevel::Medium));
        assert!(scored.blacklist);
    }

    #[test]
    fn php_empty_containers() {
        let info: AddressInfo = decode(json!({"resources": [], "balances": []}));
        assert_eq!(info.resources, Resources::default());
        assert!(info.balances.is_empty());
        let services: Services = decode(json!({"energy": {}, "bandwidth": [], "activate_address": []}));
        assert!(services.energy.is_empty() && services.bandwidth.is_empty());
        assert_eq!(services.activate_address, None);
    }

    #[test]
    fn serialization_round_trips() {
        let t: Transaction =
            decode(json!({"id": "1", "amount": 5.47, "params": {"amounts": {"energy": 65000}}}));
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(json["amount"], "5.47");
        assert_eq!(serde_json::from_value::<Transaction>(json).unwrap(), t);
        let info: AddressInfo = decode(json!({"balances": {"TRX": 12.5}}));
        assert_eq!(serde_json::to_value(&info).unwrap()["balances"], json!({"TRX": "12.5"}));
        let check: AmlCheck = decode(json!({"risk_score": null}));
        assert_eq!(serde_json::to_value(&check).unwrap()["risk_score"], Value::Null);
    }

    #[test]
    fn integers_in_other_json_types() {
        let r: Resources = decode(json!({"energy": "65000", "bandwidth": 345.0}));
        assert_eq!(r, Resources { energy: 65000, bandwidth: 345 });
        assert!(serde_json::from_value::<Resources>(json!({"energy": -1})).is_err());
        assert!(serde_json::from_value::<Resources>(json!({"energy": 1.5})).is_err());
        assert!(serde_json::from_value::<Resources>(json!({"energy": "lots"})).is_err());
    }

    #[test]
    fn wrong_shapes_are_errors() {
        assert!(serde_json::from_value::<Services>(json!({"energy": "cheap"})).is_err());
        assert!(serde_json::from_value::<AddressInfo>(json!({"resources": 5})).is_err());
        assert!(serde_json::from_value::<AmlCheck>(json!({"blacklist": "maybe"})).is_err());
    }

    #[test]
    fn services_decode() {
        let s: Services = decode(json!({
            "energy": [{"duration": 1, "min_amount": 65000, "max_amount": 10000000, "price": "0.03",
                        "price_32k": 0.96, "price_65k": "1.95", "price_131k": 3.93}],
            "bandwidth": [{"duration": 1, "min_amount": 300, "max_amount": 100000, "price": 1}],
            "activate_address": {"price": "1.4"}
        }));
        assert_eq!(s.energy[0].price.to_string(), "0.03");
        assert_eq!(s.energy[0].price_65k.to_string(), "1.95");
        assert_eq!(s.energy[0].price * Decimal::from(65000) / Decimal::from(1000), s.energy[0].price_65k);
        assert_eq!(s.bandwidth[0].price.to_string(), "1");
        assert_eq!(s.activate_address.map(|a| a.price.to_string()), Some("1.4".to_owned()));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_energy_rate_fields_mirror_amounts() {
        let rate: EnergyRate = decode(json!({"min_amount": 65000, "max_amount": "10000000"}));
        assert_eq!((rate.min_energy, rate.max_energy), (65000, 10_000_000));
        let rate: EnergyRate = decode(
            json!({"min_amount": 65000, "max_amount": 10000000, "min_energy": 32000, "max_energy": 5000000}),
        );
        assert_eq!((rate.min_amount, rate.max_amount), (65000, 10_000_000));
        assert_eq!((rate.min_energy, rate.max_energy), (65000, 10_000_000));
        let rate: EnergyRate = decode(json!({"min_energy": 32000, "max_energy": 5000000}));
        assert_eq!((rate.min_energy, rate.max_energy), (0, 0));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_energy_fields_mirror_amount() {
        let estimate: EnergyEstimate = decode(json!({"amount": 65000}));
        assert_eq!(estimate.energy, 65000);
        let estimate: EnergyEstimate = decode(json!({"amount": "65000", "energy": 32000}));
        assert_eq!((estimate.amount, estimate.energy), (65000, 65000));
        let calculation: Calculation = decode(json!({"type": "energy", "amount": 65000}));
        assert_eq!(calculation.energy, 65000);
        let calculation: Calculation = decode(json!({"type": "energy", "amount": 65000, "energy": 32000}));
        assert_eq!((calculation.amount, calculation.energy), (65000, 65000));
        assert_eq!(calculation.service, Service::Energy);
    }

    #[test]
    fn subscription_plans_keep_order_and_key() {
        let plans: SubscriptionPlans =
            serde_json::from_str(r#"{"b_plan":{"id":"2","price":"5"},"a_plan":{"id":1,"price":2.8}}"#)
                .unwrap();
        let keys: Vec<&str> = plans.0.iter().map(|p| p.subscription_id.as_str()).collect();
        assert_eq!(keys, ["b_plan", "a_plan"]);
        assert_eq!((plans.0[0].id, plans.0[1].price.to_string()), (2, "2.8".to_owned()));
        assert!(serde_json::from_str::<SubscriptionPlans>("[]").unwrap().0.is_empty());
        assert!(serde_json::from_str::<SubscriptionPlans>("true").is_err());
    }

    #[test]
    fn subscription_round_trips() {
        let sub: Subscription = decode(json!({
            "id": "sub-1", "status": "active", "total_price": 13.6, "params": [],
            "created_at": "2026-10-08T15:26:32+00:00", "stopped_at": null
        }));
        assert_eq!(sub.params, None);
        assert_eq!(sub.status, SubscriptionStatus::Active);
        let json = serde_json::to_value(&sub).unwrap();
        assert_eq!(json["total_price"], "13.6");
        assert_eq!(serde_json::from_value::<Subscription>(json).unwrap(), sub);
        let params: SubscriptionParams = decode(json!({"duration": "30", "activate_address": 1}));
        assert_eq!((params.duration_days, params.activate_address), (30, true));
        assert_eq!(serde_json::to_value(&params).unwrap()["duration"], 30);
    }

    #[test]
    fn deprecated_fields_round_trip() {
        let rate: EnergyRate = decode(json!({"duration": 1, "min_amount": 65000, "max_amount": 10000000}));
        let json = serde_json::to_value(&rate).unwrap();
        assert_eq!(json["min_energy"], 65000);
        assert_eq!(serde_json::from_value::<EnergyRate>(json).unwrap(), rate);
        let calculation: Calculation = decode(json!({"type": "bandwidth", "amount": 345, "total": "0.345"}));
        let json = serde_json::to_value(&calculation).unwrap();
        assert_eq!(json["type"], "bandwidth");
        assert_eq!(serde_json::from_value::<Calculation>(json).unwrap(), calculation);
    }
}
