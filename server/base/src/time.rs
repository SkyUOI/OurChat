pub type TimeStamp = chrono::DateTime<chrono::FixedOffset>;
pub type TimeStampUtc = chrono::DateTime<chrono::Utc>;

pub fn from_google_timestamp(ts: &prost_types::Timestamp) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
}

pub fn to_google_timestamp(ts: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: ts.timestamp(),
        nanos: ts.timestamp_subsec_nanos() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_timestamp() {
        let timestamp = chrono::Utc::now();
        assert_eq!(
            from_google_timestamp(&to_google_timestamp(timestamp)).unwrap(),
            timestamp
        );
    }
}
