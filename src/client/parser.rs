use anyhow::{Result};

use crate::client::{ SearxngResult};

#[derive(Debug)]
pub struct ContentParser {}

impl ContentParser {
    pub fn searxng(results: SearxngResult,url:String) -> Result<ParsedPage> {
        let mut content: Vec<Parts> = vec![];

        results.infoboxes.iter().for_each(|i| {
            content.push(Parts::Text(i.infobox.clone()));
            content.push(Parts::Text(i.content.clone()));
        });

        results.results.iter().for_each(|i| {
            let res = Link {
                title: i.title.clone(),
                url: i.url.clone(),
                text: i.content.clone(),
            };
            content.push(Parts::Link(res))
        });

        Ok(ParsedPage {
            title: results.query,
            content,
            url,
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
    pub content: Vec<Parts>,
}

impl ParsedPage {
    pub fn set_tab(&mut self, id: i32) {
        self.tab_id = id;
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }
}

#[derive(Debug, Clone)]
pub enum Parts {
    Text(String),
    Link(Link),
}

#[derive(Debug, Default, Clone)]
/// Used for <a> tags
pub struct Link {
    pub title: String,
    pub text: String,
    pub url: String,
}
