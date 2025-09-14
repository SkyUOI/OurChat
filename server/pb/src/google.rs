pub mod protobuf {
    use crate::time::{TimeStamp, TimeStampUtc};

    include!("./generated/google.protobuf.rs");

    // Timestamp Convert
    impl From<TimeStampUtc> for Timestamp {
        fn from(value: TimeStampUtc) -> Self {
            Self {
                seconds: value.timestamp(),
                nanos: value.timestamp_subsec_nanos() as i32,
            }
        }
    }

    impl From<TimeStamp> for Timestamp {
        fn from(value: TimeStamp) -> Self {
            Self {
                seconds: value.timestamp(),
                nanos: value.timestamp_subsec_nanos() as i32,
            }
        }
    }

    impl TryInto<TimeStampUtc> for Timestamp {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<TimeStampUtc, Self::Error> {
            chrono::DateTime::from_timestamp(self.seconds, self.nanos as u32)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))
        }
    }

    // Duration Convert

    impl From<std::time::Duration> for Duration {
        fn from(value: std::time::Duration) -> Self {
            Self {
                seconds: value.as_secs() as i64,
                nanos: value.subsec_nanos() as i32,
            }
        }
    }

    impl TryInto<std::time::Duration> for Duration {
        type Error = anyhow::Error;

        fn try_into(self) -> Result<std::time::Duration, Self::Error> {
            if self.seconds < 0 || self.nanos < 0 {
                anyhow::bail!("Duration components must be non-negative");
            }

            // Convert seconds and nanoseconds to std::time::Duration
            let std_duration = std::time::Duration::new(self.seconds as u64, self.nanos as u32);
            Ok(std_duration)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_google_timestamp() {
            let timestamp = chrono::Utc::now();
            let tmp: TimeStampUtc = Timestamp::from(timestamp).try_into().unwrap();
            assert_eq!(tmp, timestamp);
        }

        #[test]
        fn test_prost_duration() {
            let duration = std::time::Duration::from_secs(1);
            let tmp: std::time::Duration = Duration::from(duration).try_into().unwrap();
            assert_eq!(tmp, duration);
        }

        #[test]
        fn test_prost_duration_negative() {
            let duration = Duration {
                seconds: -1,
                nanos: 0,
            };
            let err: Result<std::time::Duration, _> = duration.try_into();
            claims::assert_err!(err);
        }

        #[test]
        fn test_std_duration() {
            let duration = std::time::Duration::from_secs(1);
            assert_eq!(
                Duration::from(duration),
                Duration {
                    seconds: 1,
                    nanos: 0
                }
            );
        }
    }
}
