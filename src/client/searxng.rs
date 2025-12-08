use ratatui::widgets::ListState;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::client::{
    page_part::Part,
    parser::{Link, ParsedPage, ParserTrait},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SearxngResult {
    pub query: String,
    pub results: Vec<QueryResults>,
    pub answers: Vec<SearxAnswer>,
    pub infoboxes: Vec<SearxInfo>,
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
            title: self.query.clone(),
            url: url.to_string(),
            parsed_content: content,
            state,
            ..Default::default()
        })
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
