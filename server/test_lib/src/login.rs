use crate::{create_connection, get_test_user, ClientWS};
use futures_util::{SinkExt, StreamExt};
use server::{
    connection::client_response,
    consts::MessageType,
    requests::{Login, LoginType},
};

// 测试ocid登录
async fn test_ocid_login(ocid: String) -> ClientWS {
    let user = get_test_user();
    let mut connection = create_connection().await.unwrap();
    let login_req = Login::new(ocid, user.password.clone(), LoginType::Ocid);
    connection
        .send(tungstenite::Message::Text(
            serde_json::to_string(&login_req).unwrap(),
        ))
        .await
        .unwrap();
    let ret = connection.next().await.unwrap().unwrap();
    let json: client_response::LoginResponse =
        serde_json::from_str(ret.to_text().unwrap()).unwrap();
    assert_eq!(json.code, MessageType::LoginRes);
    connection
}

async fn test_email_login(ocid: String) -> ClientWS {
    let user = get_test_user();
    let mut connection = create_connection().await.unwrap();
    let login_req = Login::new(user.email.clone(), user.password.clone(), LoginType::Email);
    connection
        .send(tungstenite::Message::Text(
            serde_json::to_string(&login_req).unwrap(),
        ))
        .await
        .unwrap();
    let ret = connection.next().await.unwrap().unwrap();
    let json: client_response::LoginResponse =
        serde_json::from_str(ret.to_text().unwrap()).unwrap();
    assert_eq!(json.code, MessageType::LoginRes);
    assert_eq!(json.ocid.unwrap(), ocid);
    connection
}

pub(crate) async fn test_login(ocid: String) -> ClientWS {
    test_ocid_login(ocid.clone())
        .await
        .close(None)
        .await
        .unwrap();
    test_email_login(ocid).await
}
