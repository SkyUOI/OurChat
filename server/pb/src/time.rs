use anyhow::bail;

pub type TimeStamp = chrono::DateTime<chrono::FixedOffset>;
pub type TimeStampUtc = chrono::DateTime<chrono::Utc>;

pub fn from_google_timestamp(
    ts: &crate::google::protobuf::Timestamp,
) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
}

pub fn to_google_timestamp(
    ts: chrono::DateTime<chrono::Utc>,
) -> crate::google::protobuf::Timestamp {
    crate::google::protobuf::Timestamp {
        seconds: ts.timestamp(),
        nanos: ts.timestamp_subsec_nanos() as i32,
    }
}

pub fn prost_duration_to_std_duration(
    prost_duration: crate::google::protobuf::Duration,
) -> anyhow::Result<std::time::Duration> {
    if prost_duration.seconds < 0 || prost_duration.nanos < 0 {
        bail!("Duration components must be non-negative");
    }

    // Convert seconds and nanoseconds to std::time::Duration
    let std_duration =
        std::time::Duration::new(prost_duration.seconds as u64, prost_duration.nanos as u32);
    Ok(std_duration)
}

pub fn std_duration_to_prost_duration(
    std_duration: std::time::Duration,
) -> crate::google::protobuf::Duration {
    // Convert std::time::Duration to pb::google::protobuf::Duration
    crate::google::protobuf::Duration {
        seconds: std_duration.as_secs() as i64,
        nanos: std_duration.subsec_nanos() as i32,
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

    #[test]
    fn test_prost_duration() {
        let duration = std::time::Duration::from_secs(1);
        assert_eq!(
            prost_duration_to_std_duration(std_duration_to_prost_duration(duration)).unwrap(),
            duration
        );
    }

    #[test]
    fn test_prost_duration_negative() {
        let duration = crate::google::protobuf::Duration {
            seconds: -1,
            nanos: 0,
        };
        assert!(prost_duration_to_std_duration(duration).is_err());
    }

    #[test]
    fn test_std_duration() {
        let duration = std::time::Duration::from_secs(1);
        assert_eq!(
            std_duration_to_prost_duration(duration),
            crate::google::protobuf::Duration {
                seconds: 1,
                nanos: 0
            }
        );
    }
}
