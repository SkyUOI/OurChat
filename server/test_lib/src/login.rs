use crate::{create_connection, get_test_user, ClientWS};
use futures_util::SinkExt;
use server::requests::{Login, LoginType};

pub(crate) async fn test_login() -> ClientWS {
    let user = get_test_user();
    let mut connection = create_connection().await.unwrap();
    let login_req = Login::new(user.ocid.clone(), user.password.clone(), LoginType::Ocid);
    connection
        .send(tungstenite::Message::Text(
            serde_json::to_string(&login_req).unwrap(),
        ))
        .await
        .unwrap();
    connection
}
