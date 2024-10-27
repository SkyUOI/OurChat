use crate::helper;
use server::{
    client::{
        MsgConvert,
        requests::{Status, upload::UploadRequest},
        response::{ErrorMsgResponse, UploadResponse},
    },
    consts::MessageType,
};
use tokio::fs::read_to_string;

#[tokio::test]
async fn test_upload() {
    let mut app = helper::TestApp::new(None).await.unwrap();
    let user = app.new_user().await.unwrap();
    // rejected
    let file = read_to_string("tests/server/test_data/file1.txt")
        .await
        .unwrap();
    let size = file.len();
    let hash = base::sha3_256(file.as_bytes());
    let req = UploadRequest::new(hash, true, size as u64);
    user.lock().await.send(req.clone().to_msg()).await.unwrap();
    ErrorMsgResponse::from_json(&user.lock().await.recv().await.unwrap().to_string()).unwrap();

    user.lock().await.ocid_login().await.unwrap();
    // accepted
    user.lock().await.send(req.clone().to_msg()).await.unwrap();
    let ret =
        UploadResponse::from_json(&user.lock().await.recv().await.unwrap().to_string()).unwrap();
    assert_eq!(ret.code, MessageType::UploadRes);
    // post file

    app.async_drop().await;
}
