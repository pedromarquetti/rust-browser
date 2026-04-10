use std::fmt::Display;

use anyhow::Result;
use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{ListItem, ListState},
};
use reqwest::Url;

use crate::{
    client::page_part::{Part, PartState},
    helpers::parse_text,
    state::ListTrait,
};

/// Trait to represent a valid parsed webpage
pub trait ParserTrait {
    fn to_parsed_page(&self, url: Url, tab_id: i32) -> Result<ParsedPage>;
}

#[derive(Debug, Clone, Default)]
pub struct ParsedPage {
    pub tab_id: i32,
    pub title: String,
    pub url: String,
    pub page_links: Vec<Link>,
    pub parsed_content: ParsedContent,
    pub linecount: usize,
    pub wordcount: usize,
    pub pos: Vec<StrPos>,
    pub curr_search_idx: u16,
    pub raw_text: String,
    pub state: ListState,
    pub page_type: PageType,
}

#[derive(Debug, Default, Clone)]
pub struct StrPos {
    pub line: usize,
    pub idx: usize,
    pub _byte: usize,
}

impl Display for StrPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, pos: {}", self.line, self.idx)
    }
}

impl ParsedPage {
    /// This func fills ParsedPage::pos with a vec of results
    pub fn get_search_pos<S>(&mut self, pattern: &S)
    where
        S: Display + ToString,
    {
        let pattern = pattern.to_string();
        let mut res: Vec<StrPos> = Vec::new();
        let mut curr_offset = 0;
        for (line_n, line) in self.raw_text.lines().enumerate() {
            let mut start = 0;
            while let Some(idx_in_line) = line[start..].find(&pattern) {
                let idx = start + idx_in_line;
                res.push(StrPos {
                    line: line_n,
                    idx: idx,
                    _byte: curr_offset + idx,
                });
                start += idx_in_line + pattern.len().max(1);
            }
            curr_offset += line.len() + "\n".len()
        }
        self.pos = res;
    }

    /// Function for wrapping the raw string and setting line/word count
    pub fn to_wrapped_string(&mut self, width: u16) {
        if self.raw_text.trim().is_empty() {
            return;
        }

        let max = width.saturating_sub(4) as usize;
        if max == 0 || self.raw_text.is_empty() {
            return;
        }

        let mut wrapped = String::new();

        let mut wordcount = 0;

        for (i, line) in self.raw_text.lines().enumerate() {
            // Preserve empty lines (paragraph breaks)
            if line.trim().is_empty() {
                if i != 0 {
                    wrapped.push('\n');
                }
                continue;
            }

            let mut curr = String::new();

            wordcount += line.split_whitespace().count();

            for word in line.split_whitespace() {
                if word.len() > max {
                    if !curr.is_empty() {
                        if !wrapped.is_empty() {
                            wrapped.push('\n');
                        }

                        wrapped.push_str(&curr);
                        curr.clear();
                    }

                    let mut start = 0;
                    while start < word.len() {
                        let end = (start + max).min(word.len());
                        let chunk = &word[start..end];

                        if !wrapped.is_empty() {
                            wrapped.push('\n');
                        }
                        wrapped.push_str(chunk);
                        start = end
                    }
                    continue;
                }

                let needs_space = !curr.is_empty();
                let next_len = curr.len() + needs_space as usize + word.len();

                if next_len <= max {
                    if needs_space {
                        curr.push(' ');
                    }
                    curr.push_str(word);
                } else {
                    if !wrapped.is_empty() {
                        wrapped.push('\n');
                    }
                    wrapped.push_str(&curr);
                    curr.clear();
                    curr.push_str(word);
                }
            }
            if !curr.is_empty() {
                if !wrapped.is_empty() {
                    wrapped.push('\n');
                }
                wrapped.push_str(&curr);
            }
        }

        self.wordcount = wordcount;
        self.linecount = wrapped.lines().count();
    }
}

#[derive(Debug, Clone)]
pub enum ParsedContent {
    PartList(Vec<Part>),
    Text(Text<'static>),
}

impl Default for ParsedContent {
    fn default() -> Self {
        Self::Text(Text::from(""))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PageType {
    Search,
    /// This represents a raw parsed HTML
    #[default]
    Raw,
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
            // parsed_content: items,
            parsed_content: ParsedContent::PartList(items),
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

impl ListTrait for Link {
    fn to_list_item(&self, width: u16) -> ratatui::widgets::ListItem<'static> {
        let width = width.saturating_sub(4) as usize;

        let mut lines = vec![];

        // lines.push(Line::from(Span::raw(self.title.clone()).bold()));

        parse_text(&mut lines, self.text.to_string(), width);

        lines.push(Line::from(
            Span::raw(format!("-> {}", self.url)).italic().cyan(),
        ));

        lines.push(Line::from(""));

        ListItem::new(Text::from(lines))
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.title, self.url)
    }
}
