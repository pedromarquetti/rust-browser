use crate::client::{parser::Page, WebClient};
use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub struct WebClientState {
    pub curr_page: Page,
    pub is_loading: bool
}

impl WebClientState {
    /// shared state to search the web
    pub async fn search(&mut self, query: String) -> Result<()> {
        let page = WebClient::search(query, self).await.context("WebClientState failed to search")?;
        self.is_loading = false;
        self.curr_page = page;
        Ok(())
    }
}
