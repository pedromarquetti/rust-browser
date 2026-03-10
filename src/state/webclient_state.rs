use crate::{
    client::{WebClientTrait, parser::ParsedPage, searxng::SearxngResult},
    config::webclient_config::AvailableSearchEngines,
};
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Default)]
pub struct WebClientState {
    pub search_provider: SearchProvider,
    pub is_loading: bool,
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
    pub async fn search(&mut self, query: String, tab_id: i32) -> Result<ParsedPage> {
        if self.search_provider.url.is_empty() || query.is_empty() {
            return Err(anyhow!(format!(
                "Search Provider URL OR query is empty!\nurl {}\n query {}",
                self.search_provider.url, query
            )));
        }

        match self.search_provider.name {
            AvailableSearchEngines::SearXNG => {
                let page = SearxngResult::new()
                    .search(query.clone(), self, tab_id)
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
