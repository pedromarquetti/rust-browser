use crate::{
    client::{WebClientTrait, fetch_url::FetchUrl, parser::ParsedPage, searxng::SearxngResult},
    config::webclient_config::AvailableSearchEngines,
};
use anyhow::{Context, Result, anyhow};
use reqwest::Url;

#[derive(Debug, Clone, Default)]
pub struct WebClientState {
    pub search_provider: SearchProvider,
    pub curr_page: ParsedPage,
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
    pub async fn fetch_url(&mut self, url: Url, tab_id: i32) -> Result<()> {
        let page = FetchUrl::default().fetch_url(url.clone(), self).await?;

        self.is_loading = false;
        self.curr_page = page;
        self.curr_page.tab_id = tab_id;
        Ok(())
    }

    /// shared state to search the web
    pub async fn search(&mut self, query: String, tab_id: i32) -> Result<()> {
        if self.search_provider.url.is_empty() || query.is_empty() {
            return Err(anyhow!(format!(
                "Search Provider URL OR query is empty!\nurl {}\n query {}",
                self.search_provider.url, query
            )));
        }

        match self.search_provider.name {
            AvailableSearchEngines::SearXNG => {
                let page = SearxngResult::new()
                    .search(query.clone(), self)
                    .await
                    .context(format!(
                        "WebClientState failed to search: \n{}\n{}",
                        self.search_provider.url, query
                    ))?;
                self.is_loading = false;
                self.curr_page = page;
                self.curr_page.tab_id = tab_id;
            }
        }

        Ok(())
    }
}
