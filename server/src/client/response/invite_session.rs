use crate::consts::{MessageType, SessionID, TimeStamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteSession {
    pub code: MessageType,
    #[serde(with = "rfc3339")]
    pub expire_timestamp: TimeStamp,
    pub session_id: SessionID,
    pub inviter_id: String,
    pub message: String,
}

pub mod rfc3339 {
    use chrono::DateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::consts::TimeStamp;

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

impl InviteSession {
    pub fn new(
        expire_timestamp: TimeStamp,
        session_id: SessionID,
        inviter_id: String,
        message: String,
    ) -> Self {
        Self {
            code: MessageType::InviteSession,
            expire_timestamp,
            session_id,
            inviter_id,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::ID;

    #[test]
    fn test_invite_session() {
        let time = chrono::Utc::now();
        let request =
            InviteSession::new(time.into(), ID(0), "test".to_string(), "test".to_string());
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            json,
            format!(
                "{{\"code\":{},\"expire_timestamp\":\"{}\",\"session_id\":0,\"inviter_id\":\"test\",\"message\":\"test\"}}",
                MessageType::InviteSession as usize,
                time.to_rfc3339()
            )
        );
        let de_request = serde_json::from_str::<InviteSession>(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&de_request).unwrap(),
            serde_json::to_string(&request).unwrap()
        );
    }
}
