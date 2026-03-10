use anyhow::{Result, anyhow};
use reqwest::{Client, IntoUrl, Response, StatusCode};

/// Generic HTTP GET func
///
/// Returns Response
pub async fn get_req<U: IntoUrl + Clone>(client: Client, url: U) -> Result<Response> {
    client
        .get(url.clone())
        .header("Accept", "text/html")
        .header("Connection", "keep-alive")
        .send()
        .await
        // .context(String::from("Error fetching url ") + url.as_str())
        .map_err(|e| {
            anyhow!(
                "Error({}): {}",
                e.status().unwrap_or(StatusCode::NOT_FOUND),
                e.to_string(),
            )
        })
}
