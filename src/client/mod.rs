use std::str::FromStr;

use crate::{
    client::{
        fetcher::get_req,
        parser::{ParsedPage, ParserTrait},
        searxng::SearxngResult,
    },
    state::webclient_state::WebClientState,
};

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Url};

pub mod fetcher;
pub mod page_part;
pub mod parser;
pub mod searxng;

#[derive(Debug)]
pub struct WebClient;

impl WebClient {
    /// SearXNG request helper func
    pub async fn search_xng(query: String, state: &mut WebClientState) -> Result<ParsedPage> {
        if state.search_provider.url.is_empty() {
            return Err(anyhow!("SearXNG URL not set!"));
        }

        let mut url = Url::from_str(state.search_provider.url.as_str()).context(format!(
            "Could not parse as URL: {}",
            state.search_provider.url
        ))?;

        url.set_path("/search");
        url.query_pairs_mut()
            .clear()
            .append_pair("q", &query)
            .append_pair("format", "json");

        state.is_loading = true;

        let client = Client::builder()
            .build()
            .context("Failed creating Client")?;

        let req = get_req(client, url.clone()).await?;

        let status = req.status();

        if !status.is_success() {
            // handler for any error codes that might occur
            return Err(anyhow!(
                "URL Returned Error! {}\n {}",
                status,
                req.text().await?
            ));
        };

        let req = req
            .json::<SearxngResult>()
            .await
            .context(format!("Error decoding JSON for url {:#?}", url))?;

        req.to_parsed_page(url)
    }
}
