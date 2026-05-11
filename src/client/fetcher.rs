use anyhow::{Result, anyhow};
use reqwest::{Client, IntoUrl, Response, StatusCode};

/// Generic HTTP GET func
///
/// Returns Response
pub async fn get_req<U: IntoUrl + Clone>(client: &Client, url: U) -> Result<Response> {
    client
        .get(url.clone())
        .header("Accept", "text/html")
        .header("Connection", "keep-alive")
        .send()
        .await
        .map_err(|e| {
            anyhow!(
                "Error({}): {}",
                e.status().unwrap_or(StatusCode::NOT_FOUND),
                e.to_string(),
            )
        })
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::time::Duration;

    use reqwest::Client;

    use crate::{client::fetcher::get_req, state::APP_USER_AGENT};

    #[tokio::test]
    async fn get_req_test() -> Result<()> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(APP_USER_AGENT)
            .build()?;

        get_req(&client, "http://example.com").await?;
        Ok(())
    }
}
