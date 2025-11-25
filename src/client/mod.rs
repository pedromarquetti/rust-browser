use std::str::FromStr;

use crate::{
    client::{
        fetcher::get_req,
        parser::{ContentParser, ParsedPage},
    },
    state::webclient_state::WebClientState,
};

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

pub mod fetcher;
pub mod page_part;
pub mod parser;

#[derive(Debug)]
pub struct WebClient {}

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

        ContentParser::searxng(req, url.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SearxngResult {
    query: String,
    results: Vec<QueryResults>,
    answers: Vec<SearxAnswer>,
    infoboxes: Vec<SearxInfo>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct SearxInfo {
    infobox: String,
    id: String,
    content: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct SearxAnswer {
    url: Option<String>,
    answer: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct QueryResults {
    url: String,
    title: String,
    content: String,
}

