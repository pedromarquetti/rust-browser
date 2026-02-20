use std::fmt::{Display, write};

use anyhow::Result;
use ratatui::{text::Text, widgets::ListState};
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
    // pub parsed_content: Vec<Part>,
    pub parsed_content: ParsedContent,
    pub linecount: usize,
    pub wordcount: usize,
    pub pos: StrPos,
    pub raw_text: String,
    pub state: ListState,
    pub page_type: PageType,
}

#[derive(Debug, Default, Clone)]
pub struct StrPos {
    line: usize,
    idx: usize,
    _byte: usize,
}

impl Display for StrPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, pos: {}", self.line, self.idx)
    }
}

impl ParsedPage {
    /// (usize, usize, usize)
    /// == (line, idx_in_line, byte_idx)
    /// "idx_in_line" represents what char in the line matches the search
    /// "byte_idx" represents the raw byte pos. of the search
    pub fn get_search_pos<S>(&mut self, pattern: S) -> Option<(usize, usize, usize)>
    where
        S: Display + ToString,
    {
        let mut curr_offset = 0;
        for (line_n, line) in self.raw_text.lines().enumerate() {
            if let Some(idx_in_line) = line.find(&pattern.to_string()) {
                self.pos = StrPos { line: line_n, idx: idx_in_line, _byte: curr_offset+idx_in_line };
                return Some((line_n, idx_in_line, curr_offset + idx_in_line));
            };
            curr_offset += line.len() + "\n".len()
        }
        None
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

#[derive(Debug, Clone, Default)]
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

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.title, self.url)
    }
}
