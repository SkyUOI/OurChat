use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use server::{consts::RequestType, requests::LoginType};

use crate::{connect_to_server_internal, get_test_user, ClientWS};

#[derive(Debug, Deserialize, Serialize)]
struct Login {
    code: RequestType,
    account: String,
    password: String,
    login_type: LoginType,
}

impl Login {
    fn new(account: String, password: String, login_type: LoginType) -> Self {
        Self {
            code: RequestType::Login,
            account,
            password,
            login_type,
        }
    }
}

pub(crate) async fn test_login() -> ClientWS {
    let user = get_test_user();
    let mut connection = connect_to_server_internal().await.unwrap();
    let login_req = Login::new(user.ocid.clone(), user.password.clone(), LoginType::Ocid);
    connection
        .send(tungstenite::Message::Text(
            serde_json::to_string(&login_req).unwrap(),
        ))
        .await
        .unwrap();
    connection
}
