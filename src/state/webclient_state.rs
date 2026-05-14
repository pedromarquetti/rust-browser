use std::{str::FromStr, sync::Arc};

use crate::{
    client::{WebClientTrait, parser::ParsedPage, searxng::SearxngResult},
    config::webclient_config::AvailableSearchEngines,
};
use anyhow::{Result, anyhow};
use reqwest::{Client, Url};

#[derive(Debug, Clone, Default)]
pub struct WebClientState {
    pub search_provider: SearchProvider,
    pub is_loading: bool,
    // lazy load web client
    pub web_client: Option<Arc<Client>>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchProvider {
    pub url: String,
    pub name: AvailableSearchEngines,
}

impl SearchProvider {
    pub fn set_url<S: ToString>(&mut self, url: S) -> Self {
        Self {
            url: url.to_string(),
            name: self.name,
        }
    }
}

impl WebClientState {
    /// shared state to search the web
    pub async fn search(
        &mut self,
        query: String,
        tab_id: i32,
        client: &Client,
    ) -> Result<ParsedPage> {
        if let Err(e) = Url::from_str(&self.search_provider.url) {
            return Err(anyhow!("Invalid URL: {e}"));
        }

        if self.search_provider.url.is_empty() || query.trim().is_empty() {
            return Err(anyhow!(format!(
                "Search Provider URL OR query is empty!\nurl {}\n query {}",
                self.search_provider.url, query
            )));
        }

        match self.search_provider.name {
            AvailableSearchEngines::SearXNG => {
                let page = SearxngResult::new()
                    .search(query.clone(), self, tab_id, client)
                    .await
                    .map_err(|err| {
                        anyhow!("WebClient search returned error: {}", err.to_string())
                    })?;
                self.is_loading = false;
                Ok(page)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::webclient_config::AvailableSearchEngines;
    use crate::state::webclient_state::SearchProvider;
    use crate::state::webclient_state::WebClientState;

    #[test]
    fn search_provider_set_url_keeps_provider() {
        let mut p = SearchProvider {
            url: "a".into(),
            name: AvailableSearchEngines::SearXNG,
        };
        let updated = p.set_url("http://localhost:8080");
        assert_eq!(updated.url, "http://localhost:8080");
        assert!(matches!(updated.name, AvailableSearchEngines::SearXNG));
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let mut ws = WebClientState::default();
        ws.search_provider.url = "invalid_url".into();

        let empty_query = ws
            .search("".into(), 0, &reqwest::Client::new())
            .await
            .unwrap_err();

        // checks if configured search provider URL is valid
        assert!(
            empty_query.to_string().contains("Invalid URL"),
            "Invalid URL check"
        );

    }

    #[tokio::test]
    async fn webclient_search_rejects_empty_query() {
        let mut ws = WebClientState::default();
        // this test ALSO validates Url validation
        // search func checks if provider url is valid
        ws.search_provider.url = "http://localhost:8080".into();

        let empty_query = ws
            .search("".into(), 0, &reqwest::Client::new())
            .await
            .unwrap_err();
        // checks if user passed "" as search query
        assert!(
            empty_query.to_string().contains("query is empty"),
            "empty query check"
        );

        let whitespace_only = ws
            .search(" ".into(), 0, &reqwest::Client::new())
            .await
            .unwrap_err();

        // checks if user passed " " as search query
        assert!(
            whitespace_only.to_string().contains("query is empty"),
            "whitespace-only check"
        );
    }
}
