// TODO:add this to document

use crate::consts::MessageType;
use base::time::TimeStamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpersResponse {
    pub code: MessageType,
    pub actions: Vec<Oper>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oper {
    #[serde(with = "base::time::rfc3339")]
    pub time: TimeStamp,
    pub oper: serde_json::Value,
}

impl Oper {
    pub fn new(time: TimeStamp, oper: serde_json::Value) -> Self {
        Self { time, oper }
    }
}

impl OpersResponse {
    pub fn new(actions: Vec<Oper>) -> Self {
        Self {
            code: MessageType::ReturnOpers,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opers_response() {
        let time: TimeStamp = chrono::Utc::now().into();
        let request = OpersResponse::new(vec![Oper::new(
            time,
            serde_json::json!(
                {
                    "a":1
                }
            ),
        )]);
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            json,
            format!(
                "{{\"code\":{},\"actions\":[{{\"time\":\"{}\",\"oper\":{{\"a\":1}}}}]}}",
                MessageType::ReturnOpers as usize,
                time.to_rfc3339()
            )
        );
        let de_request = serde_json::from_str::<OpersResponse>(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&de_request).unwrap(),
            serde_json::to_string(&request).unwrap()
        );
    }
}
