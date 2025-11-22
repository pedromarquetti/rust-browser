use crate::client::{WebClient, parser::ParsedPage};
use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub struct WebClientState {
    pub curr_page: ParsedPage,
    pub is_loading: bool,
}

impl WebClientState {
    /// shared state to search the web
    pub async fn search(&mut self, query: String, tab_id: i32) -> Result<()> {
        let page = WebClient::search(query, self)
            .await
            .context("WebClientState failed to search")?;
        self.is_loading = false;
        self.curr_page = page;
        self.curr_page.tab_id = tab_id;
        Ok(())
    }


}
