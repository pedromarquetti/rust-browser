use crate::{
    client::{
        fetcher::get_req,
        parser::{ContentParser, ParsedPage},
    },
    state::webclient_state::WebClientState,
};
use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

pub mod fetcher;
pub mod parser;

#[derive(Debug)]
pub struct WebClient {}

impl WebClient {
    pub async fn search(query: String, state: &mut WebClientState) -> Result<ParsedPage> {
        let url = String::from("https://search.phlm.dev.br/?format=json&q=");
        state.is_loading = true;
        let client = Client::builder()
            .build()
            .context("Failed creating Client")?;
        let req = get_req(
            client,
            // TODO: make this url configurable
            url.clone() + &query,
        )
        .await?
        .json::<SearxngResult>()
        .await
        .context("Error decoding JSON")?;
        ContentParser::searxng(req, url+&query)
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
