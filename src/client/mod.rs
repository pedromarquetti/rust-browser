use std::str::FromStr;

use crate::{
    client::{
        fetcher::get_req,
        parser::{ParsedPage, ParserTrait},
        searxng::SearxngResult,
    },
    state::webclient_state::WebClientState,
};

use anyhow::{Context, Result, anyhow};
use ratatui::widgets::StatefulWidget;
use reqwest::{Client, Url};

pub mod fetcher;
pub mod page_part;
pub mod parser;
pub mod searxng;

pub trait WebClientTrait {
    fn search(
        &self,
        query: String,
        state: &mut WebClientState,
    ) -> impl Future<Output = Result<ParsedPage>> + Send;

    fn fetch_url(
        &self,
        url: Url,
        state: &mut WebClientState,
    ) -> impl Future<Output = Result<ParsedPage>> + Send;
}

