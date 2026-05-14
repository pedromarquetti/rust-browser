use std::{error::Error, str::FromStr};

use crate::client::{
    fetcher::get_req,
    parser::{PageType, ParsedPage, ParserTrait},
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

    pub fn into_parsed_page(self, url: Url, tab_id: i32) -> anyhow::Result<ParsedPage> {
        let mut content: Vec<Part> =
            Vec::with_capacity(self.infoboxes.len() * 2 + self.results.len());

        for i in self.infoboxes {
            content.push(Part::text(i.infobox));
            content.push(Part::text(i.content));
        }

        for i in self.results {
            let res = Link {
                title: i.title,
                url: i.url,
                text: i.content,
            };
            content.push(Part::link(res));
        }

        let mut state = ListState::default();
        if !content.is_empty() {
            state.select(Some(0));
        }

        Ok(ParsedPage {
            tab_id,
            title: format!("{} - SearXNG", self.query),
            url: url.to_string(),
            page_type: PageType::Search,
            parsed_content: crate::client::parser::ParsedContent::PartList(content),
            state: state.into(),
            ..Default::default()
        })
    }
}

impl ParserTrait for SearxngResult {
    fn to_parsed_page(&self, url: Url, tab_id: i32) -> anyhow::Result<ParsedPage> {
        self.clone().into_parsed_page(url, tab_id)
    }
}

impl WebClientTrait for SearxngResult {
    async fn search(
        &self,
        query: String,
        state: &mut crate::state::webclient_state::WebClientState,
        tab_id: i32,
        client: &Client,
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
            .map_err(|e| return anyhow!("{:#?} {:#?}", e.to_string(), e.source()))?;

        req.into_parsed_page(url, tab_id)
    }

    async fn fetch_url(
        &self,
        _url: Url,
        _tab_id: i32,
        _client: &Client,
    ) -> anyhow::Result<ParsedPage> {
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

#[cfg(test)]
mod test {
    use crate::state::webclient_state::WebClientState;

    use crate::client::{
        parser::{PageType, ParsedContent},
        searxng::{QueryResults, SearxInfo, SearxngResult, WebClientTrait},
    };
    use reqwest::Url;

    #[test]
    fn into_parsed_page_builds_search_page() -> anyhow::Result<()> {
        let s = SearxngResult {
            query: "rust".into(),
            infoboxes: vec![SearxInfo {
                id: "1".into(),
                infobox: "Info title".into(),
                content: "Info body".into(),
            }],
            results: vec![QueryResults {
                title: "Result 1".into(),
                url: "https://example.com".into(),
                content: "Snippet".into(),
            }],
            ..Default::default()
        };

        let mut page = s.into_parsed_page(Url::parse("http://localhost:8080/search")?, 7)?;
        assert_eq!(page.page_type, PageType::Search, "page_type check");
        assert_eq!(page.title, "rust - SearXNG", "title check");

        let items = match page.parsed_content {
            ParsedContent::PartList(items) => items,
            _ => panic!("expected PartList"),
        };
        assert_eq!(items.len(), 3); // 2 from infobox + 1 result
        
        page.state.get_mut().select_next();
        assert_eq!(page.state.borrow().selected(), Some(1), "check selected item");
        Ok(())
    }

    #[tokio::test]
    async fn searx_search_fails_with_empty_provider_url() {
        let mut webclient = WebClientState::default(); // url empty
        let client = reqwest::Client::new();

        let err = SearxngResult::new()
            .search("rust".into(), &mut webclient, 0, &client)
            .await
            .unwrap_err();

        assert!(err.to_string().contains("SearXNG URL not set"));
    }
}
