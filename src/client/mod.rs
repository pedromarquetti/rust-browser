use crate::{client::parser::ParsedPage, state::webclient_state::WebClientState};

use anyhow::Result;
use reqwest::Url;

pub mod fetch_url;
pub mod fetcher;
pub mod page_part;
pub mod parser;
pub mod searxng;

pub trait WebClientTrait {
    fn search(
        &self,
        query: String,
        state: &mut WebClientState,
        tab_id: i32,
    ) -> impl Future<Output = Result<ParsedPage>> + Send;

    fn fetch_url(&self, url: Url,tab_id: i32) -> impl Future<Output = Result<ParsedPage>> + Send;
}
