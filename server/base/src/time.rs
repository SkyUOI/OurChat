pub type TimeStamp = chrono::DateTime<chrono::FixedOffset>;
pub type TimeStampUtc = chrono::DateTime<chrono::Utc>;

pub mod rfc3339 {
    use super::TimeStamp;
    use chrono::DateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &TimeStamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TimeStamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
    }
}
