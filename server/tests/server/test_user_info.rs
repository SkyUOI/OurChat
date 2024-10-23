use crate::helper::TestApp;
use server::client::basic::{GetAccountValues, SetAccountValues, SetFriendValues};
use server::client::requests::{SetAccountRequest, SetFriendInfoRequest};
use server::client::response::{ErrorMsgResponse, SetAccountInfoResponse};
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
                GetAccountValues::Ocid,
                GetAccountValues::Email,
                GetAccountValues::UserName,
            ])
            .to_msg(),
        )
        .await
        .unwrap();
    let ret =
        GetAccountInfoResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::GetAccountInfoRes);
    assert_eq!(ret.status, requests::Status::Success);
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::Ocid),
        Some(&serde_json::Value::String(user_ocid.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::UserName),
        Some(&serde_json::Value::String(user_name.clone()))
    );
    // don't have privileges
    assert_eq!(
        ret.data.unwrap().get(&GetAccountValues::Email),
        Some(&serde_json::Value::Null)
    );
    // now have privileges
    user.lock().await.ocid_login().await.unwrap();
    user.lock()
        .await
        .send(
            GetAccountInfoRequest::new(user_ocid.clone(), vec![
                GetAccountValues::Ocid,
                GetAccountValues::Email,
                GetAccountValues::UserName,
                GetAccountValues::Friends,
            ])
            .to_msg(),
        )
        .await
        .unwrap();
    let ret =
        GetAccountInfoResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::Ocid),
        Some(&serde_json::Value::String(user_ocid.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::UserName),
        Some(&serde_json::Value::String(user_name.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::Email),
        Some(&serde_json::Value::String(user_email.clone()))
    );
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::Friends),
        Some(&serde_json::Value::Array(vec![]))
    );
    // TODO:add display_name test
    app.async_drop().await;
}

#[tokio::test]
async fn test_set_user_info() {
    let mut app = TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();

    let ocid = user.lock().await.ocid.clone();

    let new_name = "test_set_user_info".to_string();
    // dont have privileges
    user.lock()
        .await
        .send(
            SetAccountRequest::new(collection_literals::collection! {
                SetAccountValues::UserName => serde_json::Value::String(new_name.clone())
            })
            .to_msg(),
        )
        .await
        .unwrap();
    // will be rejected
    let ret = ErrorMsgResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::ErrorMsg);
    // now have privileges
    user.lock().await.ocid_login().await.unwrap();
    user.lock()
        .await
        .send(
            SetAccountRequest::new(collection_literals::collection! {
                SetAccountValues::UserName => serde_json::Value::String(new_name.clone())
            })
            .to_msg(),
        )
        .await
        .unwrap();
    let ret =
        SetAccountInfoResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::SetAccountInfoRes);
    assert_eq!(ret.status, requests::Status::Success);
    // get name
    user.lock()
        .await
        .send(GetAccountInfoRequest::new(ocid.clone(), vec![GetAccountValues::UserName]).to_msg())
        .await
        .unwrap();
    let ret =
        GetAccountInfoResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(
        ret.data.as_ref().unwrap().get(&GetAccountValues::UserName),
        Some(&serde_json::Value::String(new_name.clone()))
    );
    app.async_drop().await;
}

#[tokio::test]
async fn test_set_friend_info() {
    let mut app = TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    let user2 = app.new_user().await.unwrap();
    let user_ocid = user.lock().await.ocid.clone();
    let user2_ocid = user2.lock().await.ocid.clone();
    let new_name = "xxx";
    // dont have privileges
    // try set display names for him
    user.lock()
        .await
        .send(
            SetFriendInfoRequest::new(user2_ocid.clone(), collection_literals::collection! {
                SetFriendValues::DisplayName => serde_json::Value::String(new_name.to_owned())
            })
            .to_msg(),
        )
        .await
        .unwrap();
    // will be rejected
    let ret = ErrorMsgResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::ErrorMsg);
    // now have privileges,but is no friends now
    user.lock().await.ocid_login().await.unwrap();
    user.lock()
        .await
        .send(
            SetFriendInfoRequest::new(user2_ocid.clone(), collection_literals::collection! {
                SetFriendValues::DisplayName => serde_json::Value::String(new_name.to_owned())
            })
            .to_msg(),
        )
        .await
        .unwrap();
    let ret =
        SetAccountInfoResponse::from_json(&user.lock().await.get_text().await.unwrap()).unwrap();
    assert_eq!(ret.code, MessageType::SetAccountInfoRes);
    assert_eq!(ret.status, requests::Status::ServerError);
    // add friend
    user2.lock().await.ocid_login().await.unwrap();
    app.async_drop().await;
}
