use anyhow::Result;
use ratatui::widgets::ListState;
use reqwest::Url;

use crate::client::page_part::{Part, PartState};

/// Trait to represent a valid parsed webpage
pub trait ParserTrait {
    fn to_parsed_page(&self, url: Url) -> Result<ParsedPage>;
}

#[derive(Debug, Clone, Default)]
pub struct ParsedPage {
    pub tab_id: i32,
    pub title: String,
    pub url: String,
    pub parsed_content: Vec<Part>,
    pub state: ListState,
}

impl FromIterator<(PartState, String, Link)> for ParsedPage {
    fn from_iter<T: IntoIterator<Item = (PartState, String, Link)>>(iter: T) -> Self {
        let items: Vec<Part> = iter
            .into_iter()
            .map(|(state, text, link)| {
                // creating local Part
                Part::new(state, text, link)
            })
            .collect();

        let state = ListState::default();

        Self {
            parsed_content: items,
            state,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
/// Used for <a> tags
pub struct Link {
    pub title: String,
    pub text: String,
    pub url: String,
}
