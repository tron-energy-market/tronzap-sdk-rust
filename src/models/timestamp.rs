use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A timestamp, kept as the API sent it and parsed on demand.
///
/// The API has used several encodings: RFC 3339 with `Z` or an offset such as
/// `+00:00`, a space instead of the `T`, a bare date, and Unix seconds. All of them
/// are accepted, and a time without an offset is read as UTC. An unrecognised value
/// does not fail the response: [`Timestamp::unix_timestamp`] returns `None` and the
/// text stays available from [`Timestamp::as_str`].
///
/// ```
/// use tronzap_sdk::models::Timestamp;
///
/// let created = Timestamp::new("2026-08-14T09:30:00+00:00");
/// assert_eq!(created.unix_timestamp(), Some(1_786_699_800));
///
/// let odd = Timestamp::new("yesterday");
/// assert_eq!(odd.unix_timestamp(), None);
/// assert_eq!(odd.as_str(), "yesterday");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Timestamp(String);

impl Timestamp {
    /// Wraps timestamp text without validating it.
    pub fn new(raw: impl Into<String>) -> Self {
        Timestamp(raw.into())
    }

    /// Returns the timestamp exactly as the API sent it.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the timestamp as whole seconds since the Unix epoch, or `None` when
    /// the text is in an unrecognised format.
    pub fn unix_timestamp(&self) -> Option<i64> {
        parse(self.0.trim()).map(|(seconds, _)| seconds)
    }

    /// Returns the timestamp as a [`SystemTime`], or `None` when the text is in an
    /// unrecognised format.
    pub fn to_system_time(&self) -> Option<SystemTime> {
        let (seconds, nanos) = parse(self.0.trim())?;
        if seconds >= 0 {
            UNIX_EPOCH.checked_add(Duration::new(seconds.unsigned_abs(), nanos))
        } else {
            UNIX_EPOCH
                .checked_sub(Duration::from_secs(seconds.unsigned_abs()))?
                .checked_add(Duration::from_nanos(u64::from(nanos)))
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(TimestampVisitor)
    }
}

struct TimestampVisitor;

impl Visitor<'_> for TimestampVisitor {
    type Value = Timestamp;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a timestamp string or Unix seconds")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Timestamp, E> {
        Ok(Timestamp::new(v))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Timestamp, E> {
        Ok(Timestamp(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Timestamp, E> {
        Ok(Timestamp(v.to_string()))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Timestamp, E> {
        Ok(Timestamp(v.to_string()))
    }
}

fn parse(s: &str) -> Option<(i64, u32)> {
    if !s.is_empty() && s.len() <= 12 && s.bytes().all(|b| b.is_ascii_digit()) {
        return Some((s.parse().ok()?, 0));
    }

    let bytes = s.as_bytes();
    if bytes.len() < 10 || bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }
    let year = digits(s.get(0..4)?)?;
    let month = digits(s.get(5..7)?)?;
    let day = digits(s.get(8..10)?)?;
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    if bytes.len() == 10 {
        return Some((days * 86_400, 0));
    }
    if !matches!(bytes.get(10), Some(b'T' | b't' | b' ')) {
        return None;
    }

    let rest = s.get(11..)?;
    let offset_at = rest.find(['Z', 'z', '+', '-']).unwrap_or(rest.len());
    let (clock, zone) = rest.split_at(offset_at);

    let (hms, fraction) = match clock.split_once('.') {
        Some((hms, fraction)) => (hms, Some(fraction)),
        None => (clock, None),
    };
    let mut parts = hms.split(':');
    let hour = digits(parts.next()?)?;
    let minute = digits(parts.next()?)?;
    let second = match parts.next() {
        Some(second) => digits(second)?,
        None => 0,
    };
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let nanos = match fraction {
        Some(f) if !f.is_empty() && f.len() <= 9 && f.bytes().all(|b| b.is_ascii_digit()) => {
            let padded = format!("{f:0<9}");
            padded.parse().ok()?
        }
        Some(_) => return None,
        None => 0,
    };

    let offset = match zone {
        "" | "Z" | "z" => 0,
        _ => {
            let sign = if zone.starts_with('-') { -1 } else { 1 };
            let body = zone.get(1..)?.replace(':', "");
            let (h, m) = match body.len() {
                2 => (digits(&body)?, 0),
                4 => (digits(body.get(0..2)?)?, digits(body.get(2..4)?)?),
                _ => return None,
            };
            if h > 23 || m > 59 {
                return None;
            }
            sign * (h * 3600 + m * 60)
        }
    };

    let seconds = days * 86_400 + hour * 3600 + minute * 60 + second - offset;
    Some((seconds, nanos))
}

fn digits(s: &str) -> Option<i64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Howard Hinnant's days_from_civil.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unix(s: &str) -> Option<i64> {
        Timestamp::new(s).unix_timestamp()
    }

    #[test]
    fn parses_the_formats_the_api_uses() {
        assert_eq!(unix("2026-08-14T09:30:00+00:00"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14T09:30:00Z"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14 09:30:00"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14T12:30:00+03:00"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14T04:30:00-0500"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14T09:30"), Some(1_786_699_800));
        assert_eq!(unix("2026-08-14"), Some(1_786_665_600));
        assert_eq!(unix("1786699800"), Some(1_786_699_800));
        assert_eq!(unix("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(unix("2024-02-29T00:00:00Z"), Some(1_709_164_800));
    }

    #[test]
    fn keeps_fractional_seconds() {
        let t = Timestamp::new("1970-01-01T00:00:01.25Z");
        assert_eq!(t.to_system_time(), Some(UNIX_EPOCH + Duration::from_millis(1250)));
        assert_eq!(t.unix_timestamp(), Some(1));
    }

    #[test]
    fn rejects_unrecognised_text() {
        for s in [
            "",
            "yesterday",
            "2026-13-01",
            "2023-02-29",
            "2026-08-14X09:30:00",
            "2026-08-14T25:00:00",
            "2026-08-14T09:30:00+25:00",
            "2026-08-14T09:30:00.Z",
            "2026-08-14T09:30:00:00",
        ] {
            assert_eq!(unix(s), None, "{s}");
        }
    }

    #[test]
    fn decodes_strings_and_integers() {
        let t: Timestamp = serde_json::from_str("1786699800").unwrap();
        assert_eq!(t.as_str(), "1786699800");
        let t: Timestamp = serde_json::from_str(r#""2026-08-14 09:30:00""#).unwrap();
        assert_eq!(t.to_string(), "2026-08-14 09:30:00");
        assert_eq!(serde_json::to_string(&t).unwrap(), r#""2026-08-14 09:30:00""#);
    }
}
