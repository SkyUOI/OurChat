mod test_lib;

#[tokio::test]
async fn test_status() {
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:7778/v1/status")
        .send()
        .await
        .expect("failed");
    assert!(response.status().is_success(), "{:?}", response.status());
    assert_eq!(response.content_length(), Some(0));
}
