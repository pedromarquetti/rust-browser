use std::{
    cell::{Cell, RefCell},
    fmt::Display,
    ops::Range,
};

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

const MAX_SEARCH_HITS: usize = 5000;

#[derive(Debug, Clone, Default)]
pub struct ParsedPage {
    pub tab_id: i32,
    pub title: String,
    pub url: String,
    pub page_links: Vec<Link>,
    pub parsed_content: ParsedContent,
    pub linecount: Cell<Option<usize>>,
    pub wordcount: Cell<Option<usize>>,
    pub wrap_cache: RefCell<Option<WrapCache>>,
    pub pos: RefCell<Vec<StrPos>>,
    pub curr_search_idx: Cell<u16>,
    pub raw_text: String,
    pub state: RefCell<ListState>,
    pub list_items_cache: RefCell<Option<(u16, Vec<ListItem<'static>>)>>,
    pub page_type: PageType,
}

#[derive(Debug, Clone, Default)]
/// used to (not) rewrap if not needed
/// (if cached -> no need to rewrap)
pub struct WrapCache {
    pub width: u16,
    pub line_ranges: Vec<Range<usize>>,
    pub line_start_bytes: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct StrPos {
    pub line: usize,
    pub idx: usize,
    pub str_byte: usize,
}

impl Display for StrPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {}, pos: {}", self.line, self.idx)
    }
}

impl ParsedPage {
    /// This func fills ParsedPage::pos with a vec of results
    pub fn get_search_pos<S>(&self, pattern: &S)
    where
        S: Display + ToString,
    {
        let pattern = pattern.to_string();
        if pattern.is_empty() {
            self.pos.borrow_mut().clear();
            return;
        }

        let mut res: Vec<StrPos> = Vec::new();
        let mut curr_offset = 0;
        'lines: for (line_n, line) in self.raw_text.lines().enumerate() {
            let mut start = 0;
            while let Some(idx_in_line) = line[start..].find(&pattern) {
                if res.len() >= MAX_SEARCH_HITS {
                    // break for loop, not while loop if we reach MAX_SEARCH
                    break 'lines;
                }

                let idx = start + idx_in_line;
                res.push(StrPos {
                    line: line_n,
                    idx: idx,
                    str_byte: curr_offset + idx,
                });
                start += idx_in_line + pattern.len().max(1);
            }
            curr_offset += line.len() + "\n".len()
        }
        let mut pos = self.pos.borrow_mut();
        *pos = res;
    }

    /// Updates wrapping metadata used for rendering and search navigation.
    pub fn to_wrapped_string(&self, width: u16) {
        let wrap_width = width.max(1);
        let needs_rebuild = self
            .wrap_cache
            .borrow()
            .as_ref()
            .map(|cache| cache.width != wrap_width)
            .unwrap_or(true);
        if !needs_rebuild {
            return;
        }

        let max = wrap_width as usize;
        let mut line_ranges = Vec::new();
        let mut line_start_bytes = Vec::new();
        let mut wordcount = 0;
        let mut line_offset = 0;

        for line in self.raw_text.lines() {
            wordcount += line.split_whitespace().count();

            if line.is_empty() {
                line_ranges.push(line_offset..line_offset);
                line_start_bytes.push(line_offset);
                line_offset += "\n".len();
                continue;
            }

            let mut start = 0;
            while start < line.len() {
                let mut end = (start + max).min(line.len());
                while end > start && !line.is_char_boundary(end) {
                    end -= 1;
                }

                if end == start {
                    if let Some(next_char) = line[start..].chars().next() {
                        end = start + next_char.len_utf8();
                    } else {
                        break;
                    }
                }

                let abs_start = line_offset + start;
                let abs_end = line_offset + end;
                line_ranges.push(abs_start..abs_end);
                line_start_bytes.push(abs_start);
                start = end;
            }

            line_offset += line.len() + "\n".len();
        }

        self.wordcount.set(Some(wordcount));
        self.linecount.set(Some(line_ranges.len()));

        *self.wrap_cache.borrow_mut() = Some(WrapCache {
            width: wrap_width,
            line_ranges,
            line_start_bytes,
        });
    }

    pub fn wrapped_lines(&self, width: u16) -> Vec<Line<'_>> {
        self.to_wrapped_string(width);
        let cache = self.wrap_cache.borrow();
        let Some(cache) = cache.as_ref() else {
            return vec![Line::from("")];
        };

        cache
            .line_ranges
            .iter()
            .map(|range| Line::from(&self.raw_text[range.start..range.end]))
            .collect()
    }

    /// gets line that match search
    pub fn visual_line_for_byte(&self, width: u16, byte: usize) -> usize {
        self.to_wrapped_string(width);
        let cache = self.wrap_cache.borrow();
        let Some(cache) = cache.as_ref() else {
            return 0;
        };

        cache
            .line_start_bytes
            .partition_point(|&start| start <= byte)
            .saturating_sub(1)
    }

    /// Sets list items as cache
    pub fn search_items_cache(&self, width: u16) {
        let needs_rebuild = self
            .list_items_cache
            .borrow()
            .as_ref()
            .map(|(cached_width, _)| *cached_width != width)
            .unwrap_or(true);

        // return early if list items didn't change'
        if !needs_rebuild {
            return;
        }

        let items = match &self.parsed_content {
            ParsedContent::PartList(list) => {
                // gets list items 
                list.iter().map(|part| part.to_list_item(width)).collect()
            }
            ParsedContent::Text(_) => Vec::new(),
            ParsedContent::Empty => Vec::new(),
        };

        // sets list items as cache
        *self.list_items_cache.borrow_mut() = Some((width, items));
    }

    pub fn search_items(&self) -> Vec<ListItem<'static>> {
        self.list_items_cache
            .borrow()
            .as_ref()
            .map(|(_, items)| items.clone())
            .unwrap_or_default()
    }

    pub fn search_items_len(&self) -> usize {
        self.list_items_cache
            .borrow()
            .as_ref()
            .map(|(_, items)| items.len())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default)]
pub enum ParsedContent {
    PartList(Vec<Part>),
    Text(Text<'static>),
    #[default]
    Empty,
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
            state: RefCell::new(state),
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
