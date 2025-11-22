use anyhow::Result;
use ratatui::widgets::{ListItem, ListState};

use crate::client::SearxngResult;
use crate::client::page_part::{Part, PartState};

#[derive(Debug)]
pub struct ContentParser {}

impl ContentParser {
    pub fn searxng(results: SearxngResult, url: String) -> Result<ParsedPage> {
        let mut content: Vec<Part> = vec![];

        results.infoboxes.iter().for_each(|i| {
            content.push(Part::text(i.infobox.clone()));
            content.push(Part::text(i.content.clone()));
        });

        results.results.iter().for_each(|i| {
            let res = Link {
                title: i.title.clone(),
                url: i.url.clone(),
                text: i.content.clone(),
            };
            content.push(Part::link(res))
        });

        let state = ListState::default();

        Ok(ParsedPage {
            title: results.query,
            parsed_content: content,
            url,
            state,
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Pages {
    pub page_list: Vec<ParsedPage>,
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

impl ParsedPage {
    pub fn set_tab(&mut self, id: i32) {
        self.tab_id = id;
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
}

#[derive(Debug, Default, Clone)]
/// Used for <a> tags
pub struct Link {
    pub title: String,
    pub text: String,
    pub url: String,
}
