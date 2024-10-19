use crate::helper::TestApp;
use server::client::basic::RequestValues;
use server::{
    client::{
        MsgConvert,
        requests::{self, GetAccountInfoRequest},
        response::GetAccountInfoResponse,
    },
    consts::MessageType,
};

#[tokio::test]
async fn test_get_user_info() {
    let mut app = TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    // request before logged in
    let user_ocid = user.lock().await.ocid.clone();
    let user_name = user.lock().await.name.clone();
    let user_email = user.lock().await.email.clone();

    user.lock()
        .await
        .send(
            GetAccountInfoRequest::new(user_ocid.clone(), vec![
                RequestValues::Ocid,
                RequestValues::Email,
                RequestValues::UserName,
            ])
            .to_msg(),
        )
        .await
        .unwrap();
    let ret: GetAccountInfoResponse =
        serde_json::from_str(user.lock().await.get().await.unwrap().to_text().unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::GetAccountInfoRes);
    assert_eq!(ret.status, requests::Status::Success);
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::Ocid),
        Some(&serde_json::Value::String(user_ocid.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::UserName),
        Some(&serde_json::Value::String(user_name.clone()))
    );
    // don't have privileges
    assert_eq!(
        ret.data.unwrap().get(&RequestValues::Email),
        Some(&serde_json::Value::Null)
    );
    // now have privileges
    user.lock().await.ocid_login().await.unwrap();
    user.lock()
        .await
        .send(
            GetAccountInfoRequest::new(user_ocid.clone(), vec![
                RequestValues::Ocid,
                RequestValues::Email,
                RequestValues::UserName,
                RequestValues::Friends,
            ])
            .to_msg(),
        )
        .await
        .unwrap();
    let ret: GetAccountInfoResponse =
        serde_json::from_str(user.lock().await.get().await.unwrap().to_text().unwrap()).unwrap();
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::Ocid),
        Some(&serde_json::Value::String(user_ocid.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::UserName),
        Some(&serde_json::Value::String(user_name.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::Email),
        Some(&serde_json::Value::String(user_email.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&RequestValues::Friends),
        Some(&serde_json::Value::Array(vec![]))
    );
    // TODO:add display_name test
    app.async_drop().await;
}
