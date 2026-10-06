// The API is backed by PHP: an empty object can arrive as `[]`, an empty list as
// `{}`, and scalars in the "wrong" JSON type. These helpers accept every form seen.
// Decimals are (de)serialized here rather than through rust_decimal's serde
// features, whose output format another crate in the build could switch.

use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::models::{Decimal, Timestamp};

fn value<'de, D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
    Value::deserialize(d)
}

fn is_blank(v: &Value) -> bool {
    match v {
        Value::Null => true,
        Value::String(s) => s.trim().is_empty(),
        _ => false,
    }
}

fn convert<T: DeserializeOwned, E: Error>(v: Value) -> Result<T, E> {
    T::deserialize(v).map_err(E::custom)
}

pub(crate) fn string<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    match value(d)? {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        other => Err(D::Error::custom(format!("expected a string, found {other}"))),
    }
}

pub(crate) fn opt_string<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let s = string(d)?;
    Ok(if s.is_empty() { None } else { Some(s) })
}

pub(crate) fn u64<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let v = value(d)?;
    if is_blank(&v) {
        return Ok(0);
    }
    let parsed = match &v {
        Value::Number(n) => n.as_u64().or_else(|| n.as_f64().and_then(whole)),
        Value::String(s) => {
            let s = s.trim();
            s.parse().ok().or_else(|| s.parse().ok().and_then(whole))
        }
        _ => None,
    };
    parsed.ok_or_else(|| D::Error::custom(format!("expected a non-negative integer, found {v}")))
}

fn whole(f: f64) -> Option<u64> {
    (f.fract() == 0.0 && (0.0..=u64::MAX as f64).contains(&f)).then_some(f as u64)
}

pub(crate) fn u32<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    let n = u64(d)?;
    u32::try_from(n).map_err(|_| D::Error::custom(format!("{n} is out of range")))
}

pub(crate) fn bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    match value(d)? {
        Value::Null => Ok(false),
        Value::Bool(b) => Ok(b),
        Value::Number(n) => Ok(n.as_f64().is_some_and(|f| f != 0.0)),
        Value::String(s) => match s.trim() {
            "true" | "1" => Ok(true),
            "" | "false" | "0" => Ok(false),
            other => Err(D::Error::custom(format!("expected a boolean, found {other:?}"))),
        },
        other => Err(D::Error::custom(format!("expected a boolean, found {other}"))),
    }
}

pub(crate) fn object<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    match value(d)? {
        Value::Null => Ok(T::default()),
        Value::Array(a) if a.is_empty() => Ok(T::default()),
        v => convert(v),
    }
}

pub(crate) fn opt_object<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    match value(d)? {
        Value::Null => Ok(None),
        Value::Array(a) if a.is_empty() => Ok(None),
        v => convert(v).map(Some),
    }
}

pub(crate) fn list<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    match value(d)? {
        Value::Null => Ok(Vec::new()),
        Value::Object(o) if o.is_empty() => Ok(Vec::new()),
        v => convert(v),
    }
}

pub(crate) fn opt_timestamp<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Timestamp>, D::Error> {
    let v = value(d)?;
    if is_blank(&v) { Ok(None) } else { convert(v).map(Some) }
}

pub(crate) fn wire_enum<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: From<String>,
{
    string(d).map(T::from)
}

pub(crate) fn opt_wire_enum<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: From<String>,
{
    opt_string(d).map(|s| s.map(T::from))
}

pub(crate) fn unknown<T: From<String>>() -> T {
    T::from(String::new())
}

fn to_decimal(v: &Value) -> Result<Decimal, String> {
    let text = match v {
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                return Ok(Decimal::from(u));
            }
            if let Some(i) = n.as_i64() {
                return Ok(Decimal::from(i));
            }
            n.as_f64().map(|f| f.to_string()).ok_or_else(|| format!("{n} is not a decimal number"))?
        }
        Value::String(s) => s.trim().to_owned(),
        other => return Err(format!("expected a decimal number, found {other}")),
    };
    Decimal::from_str_exact(&text)
        .or_else(|_| Decimal::from_scientific(&text))
        .map_err(|_| format!("{text:?} is not a decimal number"))
}

pub(crate) mod decimal {
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    use super::{is_blank, to_decimal, value};
    use crate::models::Decimal;

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
        let v = value(d)?;
        if is_blank(&v) { Ok(Decimal::ZERO) } else { to_decimal(&v).map_err(D::Error::custom) }
    }

    pub(crate) fn serialize<S: Serializer>(v: &Decimal, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(v)
    }
}

pub(crate) mod opt_decimal {
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    use super::{is_blank, to_decimal, value};
    use crate::models::Decimal;

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Decimal>, D::Error> {
        let v = value(d)?;
        if is_blank(&v) { Ok(None) } else { to_decimal(&v).map(Some).map_err(D::Error::custom) }
    }

    pub(crate) fn serialize<S: Serializer>(v: &Option<Decimal>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(d) => s.collect_str(d),
            None => s.serialize_none(),
        }
    }
}

pub(crate) mod decimal_map {
    use std::collections::BTreeMap;

    use serde::de::Error;
    use serde::{Deserializer, Serializer};
    use serde_json::Value;

    use super::{to_decimal, value};
    use crate::models::Decimal;

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<BTreeMap<String, Decimal>, D::Error> {
        match value(d)? {
            Value::Null => Ok(BTreeMap::new()),
            Value::Array(a) if a.is_empty() => Ok(BTreeMap::new()),
            Value::Object(o) => {
                o.into_iter().map(|(k, v)| to_decimal(&v).map(|d| (k, d)).map_err(D::Error::custom)).collect()
            }
            other => Err(D::Error::custom(format!("expected an object, found {other}"))),
        }
    }

    pub(crate) fn serialize<S: Serializer>(m: &BTreeMap<String, Decimal>, s: S) -> Result<S::Ok, S::Error> {
        s.collect_map(m.iter().map(|(k, v)| (k, v.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::{Value, json};

    use crate::models::Decimal;

    #[derive(Deserialize)]
    struct Price {
        #[serde(default, with = "super::decimal")]
        price: Decimal,
    }

    fn price(v: Value) -> Result<Decimal, serde_json::Error> {
        serde_json::from_value::<Price>(json!({ "price": v })).map(|p| p.price)
    }

    fn d(s: &str) -> Decimal {
        Decimal::from_str_exact(s).unwrap()
    }

    #[test]
    fn numbers_and_strings_decode_alike() {
        assert_eq!(price(json!(5.47)).unwrap(), d("5.47"));
        assert_eq!(price(json!("5.47")).unwrap(), d("5.47"));
        assert_eq!(price(json!(0.0841)).unwrap().to_string(), "0.0841");
        assert_eq!(price(json!(" 0.0841 ")).unwrap().to_string(), "0.0841");
        assert_eq!(price(json!(65000)).unwrap(), Decimal::from(65000));
        assert_eq!(price(json!(-3)).unwrap(), Decimal::from(-3));
        assert_eq!(price(json!("1e-3")).unwrap(), d("0.001"));
    }

    #[test]
    fn compares_numerically() {
        assert_eq!(price(json!("1.50")).unwrap(), price(json!(1.5)).unwrap());
        assert_eq!(price(json!("1.50")).unwrap().to_string(), "1.50");
    }

    #[test]
    fn exact_arithmetic() {
        let p = price(json!("0.0841")).unwrap();
        assert_eq!(p * Decimal::from(65000), d("5466.5"));
        assert_eq!(price(json!(0.1)).unwrap() + price(json!(0.2)).unwrap(), d("0.3"));
    }

    #[test]
    fn blank_is_zero() {
        assert_eq!(price(Value::Null).unwrap(), Decimal::ZERO);
        assert_eq!(price(json!("")).unwrap(), Decimal::ZERO);
        assert_eq!(serde_json::from_value::<Price>(json!({})).unwrap().price, Decimal::ZERO);
    }

    #[test]
    fn rejects_non_numeric_and_out_of_range() {
        for v in [json!("abc"), json!("inf"), json!("NaN"), json!(true), json!("1.2.3"), json!([1])] {
            assert!(price(v.clone()).is_err(), "{v}");
        }
        assert!(price(json!("1e40")).is_err());
        assert!(price(json!("0.12345678901234567890123456789")).is_err());
    }
}
