use std::str::FromStr;

use crate::{
    client::{
        fetcher::get_req,
        parser::{PageType, ParsedPage, ParserTrait},
    },
};

use anyhow::{Context, anyhow, bail};
use reqwest::{Client, Url};

use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

use crate::client::{WebClientTrait, page_part::Part, parser::Link};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
/// Main struct for handling SearXNG search engine
pub struct SearxngResult {
    pub query: String,
    pub results: Vec<QueryResults>,
    pub answers: Vec<SearxAnswer>,
    pub infoboxes: Vec<SearxInfo>,
}

impl SearxngResult {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ParserTrait for SearxngResult {
    fn to_parsed_page(&self, url: Url) -> anyhow::Result<ParsedPage> {
        let mut content: Vec<Part> = vec![];

        self.infoboxes.iter().for_each(|i| {
            content.push(Part::text(i.infobox.clone()));
            content.push(Part::text(i.content.clone()));
        });

        self.results.iter().for_each(|i| {
            let res = Link {
                title: i.title.clone(),
                url: i.url.clone(),
                text: i.content.clone(),
            };
            content.push(Part::link(res))
        });

        let mut state = ListState::default();

        if !content.is_empty() {
            state.select(Some(0));
        }

        Ok(ParsedPage {
            title: self.query.clone() + " - SearXNG",
            url: url.to_string(),
            page_type: PageType::Search,
            parsed_content: crate::client::parser::ParsedContent::PartList(content),
            state,
            ..Default::default()
        })
    }
}

impl WebClientTrait for SearxngResult {
    async fn search(
        &self,
        query: String,
        state: &mut crate::state::webclient_state::WebClientState,
    ) -> anyhow::Result<ParsedPage> {
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

    async fn fetch_url(&self, _url: Url) -> anyhow::Result<ParsedPage> {
        bail!("This provider does not implement direct url!")
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SearxInfo {
    pub infobox: String,
    pub id: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SearxAnswer {
    pub url: Option<String>,
    pub answer: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct QueryResults {
    pub url: String,
    pub title: String,
    pub content: String,
}
