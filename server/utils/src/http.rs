use reqwest::{Client, Error, redirect};
use std::time::Duration;

pub async fn test_and_get_http_status(url_without_scheme: impl AsRef<str>) -> Result<bool, Error> {
    let http_url = format!("http://{}", url_without_scheme.as_ref());

    // Create client, set timeout and redirect policy
    let client = Client::builder()
        .redirect(redirect::Policy::limited(5))
        .timeout(Duration::from_secs(10))
        .build()?;

    // Send http request
    let response = client.get(&http_url).send().await?;

    // Check if the scheme is "https"
    let final_url = response.url();
    if final_url.scheme() == "https" {
        return Ok(true);
    }

    // test HTTPS directly
    let https_url = http_url.replace("http://", "https://");
    match client.get(&https_url).send().await {
        Ok(_resp) => Ok(true),
        Err(e) => Err(e),
    }
}
