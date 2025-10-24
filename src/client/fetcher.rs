use anyhow::{Context, Result};
use reqwest::{Client, IntoUrl, Response};


/// Generic HTTP GET func
///
/// Returns Response
pub async fn get_req<U: IntoUrl>(client: Client, url: U) -> Result<Response> {
    Ok(client
        .get(url.as_str())
        .header("Accept", "text/html")
        .header("Connection", "keep-alive")
        .send()
        .await
        .context(String::from("Error fetching url ") + url.as_str())?)
}
