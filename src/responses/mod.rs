//! Results of the API calls.
//!
//! Collections are never missing: an absent list decodes as empty. Values the API
//! may omit are `Option`s. All structs are `#[non_exhaustive]`, so fields can be
//! added in minor releases.

mod de;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::{
    AmlCheckType, AmlDirection, AmlRiskLevel, AmlStatus, Decimal, Service, Timestamp, TransactionStatus,
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
#[non_exhaustive]
pub struct EnergyRate {
    /// The rental duration in hours.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration: u32,
    /// The smallest amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub min_amount: u64,
    /// The largest amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub max_amount: u64,
    /// The smallest energy amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub min_energy: u64,
    /// The largest energy amount in this tier.
    #[serde(default, deserialize_with = "de::u64")]
    pub max_energy: u64,
    /// The price of **one unit** of energy. Bandwidth is priced per 1000 units
    /// instead, see [`BandwidthRate::price`].
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
    /// costs 0.345. Energy is priced per single unit instead.
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
#[non_exhaustive]
pub struct EnergyEstimate {
    /// The estimated resource amount.
    #[serde(default, deserialize_with = "de::u64")]
    pub amount: u64,
    /// The estimated energy. Buy this much to cover the transfer.
    #[serde(default, deserialize_with = "de::u64")]
    pub energy: u64,
    /// The rental duration in hours the price refers to.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration: u32,
    /// The cost of the energy.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
    /// The address activation fee included in `total`, if any.
    #[serde(default, with = "de::decimal")]
    pub activation_fee: Decimal,
    /// The total cost.
    #[serde(default, with = "de::decimal")]
    pub total: Decimal,
    /// The sender address.
    #[serde(default, deserialize_with = "de::string")]
    pub from_address: String,
    /// The recipient address.
    #[serde(default, deserialize_with = "de::string")]
    pub to_address: String,
    /// The token contract the estimate refers to.
    #[serde(default, deserialize_with = "de::string")]
    pub contract_address: String,
}

/// The price of a purchase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Calculation {
    /// The address the price was calculated for.
    #[serde(default, deserialize_with = "de::string")]
    pub address: String,
    /// The priced service.
    #[serde(rename = "type", default = "de::unknown", deserialize_with = "de::wire_enum")]
    pub service: Service,
    /// The priced resource amount.
    #[serde(default, deserialize_with = "de::u64")]
    pub amount: u64,
    /// The priced energy amount.
    #[serde(default, deserialize_with = "de::u64")]
    pub energy: u64,
    /// The rental duration in hours.
    #[serde(default, deserialize_with = "de::u32")]
    pub duration: u32,
    /// The cost of the resources.
    #[serde(default, with = "de::decimal")]
    pub price: Decimal,
    /// The address activation fee included in `total`, if any.
    #[serde(default, with = "de::decimal")]
    pub activation_fee: Decimal,
    /// The total cost.
    #[serde(default, with = "de::decimal")]
    pub total: Decimal,
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
    /// The price of one unit of energy.
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
            "energy": [{"duration": 1, "min_energy": 32000, "max_energy": 10000000, "price": "0.0841",
                        "price_32k": 2.69, "price_65k": "5.47", "price_131k": 11.02}],
            "bandwidth": [{"duration": 1, "min_amount": 300, "max_amount": 100000, "price": 1}],
            "activate_address": {"price": "1.4"}
        }));
        assert_eq!(s.energy[0].price.to_string(), "0.0841");
        assert_eq!(s.energy[0].price_65k.to_string(), "5.47");
        assert_eq!(s.bandwidth[0].price.to_string(), "1");
        assert_eq!(s.activate_address.map(|a| a.price.to_string()), Some("1.4".to_owned()));
    }
}
