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
                e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                e,
            )
        })
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use std::time::Duration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        task::JoinHandle,
        time::timeout,
    };

    use reqwest::Client;

    use crate::{client::fetcher::get_req, state::APP_USER_AGENT};

    async fn server(body: &'static str) -> Result<(String, JoinHandle<Result<()>>)> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        println!("listening on addr {:?}", listener);
        let addr = listener.local_addr()?;

        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await?;

            // Read request bytes (only to consume incoming data; not strictly  required).
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf).await?;

            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );

            stream.write_all(resp.as_bytes()).await?;
            stream.shutdown().await?;

            Ok(())
        });

        Ok((format!("http://{}", addr), handle))
    }

    #[tokio::test]
    async fn get_req_test() -> Result<()> {
        let (u, server) = server("<html>oi</html>").await?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(APP_USER_AGENT)
            .build()?;

        get_req(&client, u).await?;

        timeout(Duration::from_secs(2), server).await???; // ensures no hanging server 
        Ok(())
    }
}
