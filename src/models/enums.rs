use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! wire_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $wire:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// A value this SDK version does not know, kept verbatim.
            Unknown(String),
        }

        impl $name {
            /// Returns the value as the API spells it.
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $wire, )+
                    Self::Unknown(value) => value,
                }
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                match value {
                    $( $wire => Self::$variant, )+
                    other => Self::Unknown(other.to_owned()),
                }
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                match value.as_str() {
                    $( $wire => Self::$variant, )+
                    _ => Self::Unknown(value),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                String::deserialize(deserializer).map(Self::from)
            }
        }
    };
}

wire_enum! {
    /// A service sold through the transaction endpoint.
    ///
    /// ```
    /// use tronzap_sdk::models::Service;
    ///
    /// assert_eq!(Service::from("resource_bundle"), Service::ResourceBundle);
    /// assert_eq!(Service::Energy.as_str(), "energy");
    /// assert_eq!(Service::from("staking"), Service::Unknown("staking".into()));
    /// ```
    Service {
        /// Energy rental.
        Energy => "energy",
        /// Bandwidth rental.
        Bandwidth => "bandwidth",
        /// Energy and bandwidth in one transaction.
        ResourceBundle => "resource_bundle",
        /// TRON address activation.
        ActivateAddress => "activate_address",
    }
}

wire_enum! {
    /// The state of a transaction: `New` → `Pending` → `Success` or `Failed`.
    TransactionStatus {
        /// Created, processing has not started.
        New => "new",
        /// Being processed.
        Pending => "pending",
        /// Completed successfully.
        Success => "success",
        /// Failed.
        Failed => "failed",
    }
}

wire_enum! {
    /// The state of a subscription.
    SubscriptionStatus {
        /// Created, not started yet.
        New => "new",
        /// Being started.
        Pending => "pending",
        /// Could not be started.
        Error => "error",
        /// Delegating energy.
        Active => "active",
        /// Stopped.
        Stopped => "stopped",
        /// Ran out of time or transactions.
        Expired => "expired",
    }
}

wire_enum! {
    /// What an AML check screens.
    AmlCheckType {
        /// A wallet address.
        Address => "address",
        /// A transaction hash.
        Hash => "hash",
    }
}

wire_enum! {
    /// Which side of a screened transaction you are on. The risk is scored for the
    /// counterparty.
    AmlDirection {
        /// The funds were sent to your address: the screened address is yours and
        /// the sender is scored.
        Deposit => "deposit",
        /// You sent the funds: the screened address is the external recipient's,
        /// and the recipient is scored.
        Withdrawal => "withdrawal",
    }
}

wire_enum! {
    /// The state of an AML check.
    AmlStatus {
        /// Queued.
        Pending => "pending",
        /// Running.
        Processing => "processing",
        /// Results are available.
        Completed => "completed",
        /// Could not be completed.
        Failed => "failed",
    }
}

wire_enum! {
    /// The risk level an AML check assigned.
    AmlRiskLevel {
        /// Low risk.
        Low => "low",
        /// Medium risk.
        Medium => "medium",
        /// High risk.
        High => "high",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_round_trip() {
        for value in ["energy", "bandwidth", "resource_bundle", "activate_address"] {
            assert_eq!(Service::from(value).as_str(), value);
            assert!(!matches!(Service::from(value), Service::Unknown(_)));
        }
    }

    #[test]
    fn unknown_values_are_kept() {
        let status: TransactionStatus = serde_json::from_str(r#""refunded""#).unwrap();
        assert_eq!(status, TransactionStatus::Unknown("refunded".into()));
        assert_eq!(serde_json::to_string(&status).unwrap(), r#""refunded""#);
    }

    #[test]
    fn serializes_as_wire_string() {
        assert_eq!(serde_json::to_string(&AmlDirection::Withdrawal).unwrap(), r#""withdrawal""#);
        assert_eq!(AmlRiskLevel::from(String::from("high")), AmlRiskLevel::High);
        assert_eq!(AmlStatus::Completed.to_string(), "completed");
        assert_eq!(AmlCheckType::from("hash"), AmlCheckType::Hash);
        assert_eq!(SubscriptionStatus::from("expired"), SubscriptionStatus::Expired);
        assert_eq!(SubscriptionStatus::from("paused"), SubscriptionStatus::Unknown("paused".into()));
    }
}
