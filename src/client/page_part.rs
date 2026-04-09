use std::vec;

use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
    widgets::ListItem,
};

use crate::{client::parser::Link, helpers::parse_text, state::ListTrait};

#[derive(Debug, Clone)]
/// This struct represents a Page part:
/// Link Block, Text Block...
pub struct Part {
    state: PartState,
    pub title: Option<String>,
    pub content: Option<Content>,
    pub link: Option<Link>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartState {
    Link,
    #[default]
    Text,
}

impl Default for Part {
    fn default() -> Self {
        Self {
            state: PartState::Text,
            title: None,
            content: None,
            link: None,
        }
    }
}

impl From<&Part> for ListItem<'_> {
    fn from(value: &Part) -> Self {
        let text = match value.state {
            PartState::Text => {
                let mut lines = vec![];

                if let Some(title) = &value.title
                    && !title.is_empty()
                {
                    lines.push(Line::from(Span::raw(title.clone()).bold()));
                }

                // Add text content (will wrap)
                if let Some(content) = &value.content
                    && !content.text.is_empty()
                {
                    lines.push(Line::from(content.text.clone()));
                }

                // Add empty line for spacing
                lines.push(Line::from(""));

                Text::from(lines)
            }
            PartState::Link => {
                let link = value.link.clone().unwrap_or_default();
                let mut lines = vec![];

                // Add title if present
                if let Some(title) = &value.title
                    && !title.is_empty()
                {
                    lines.push(Line::from(Span::raw(title.clone()).bold().blue()));
                }

                // Add link text
                if let Some(content) = &value.content
                    && !content.text.is_empty()
                {
                    lines.push(Line::from(content.text.clone()));
                }

                // Add URL
                if !link.url.is_empty() {
                    lines.push(Line::from(
                        Span::raw(format!("→ {}", link.url)).italic().cyan(),
                    ));
                }

                // Add empty line for spacing
                lines.push(Line::from(""));

                Text::from(lines)
            }
        };

        ListItem::new(text)
    }
}

impl Part {
    pub fn new(state: PartState, text: String, link: Link) -> Self {
        match state {
            PartState::Link => Part::link(link),
            PartState::Text => Part::text(text),
        }
    }

    pub fn text(text: String) -> Self {
        Self {
            content: Some(Content::new(text)),
            ..Default::default()
        }
    }

    pub fn link(link: Link) -> Self {
        Self {
            content: Some(Content::new(link.text.clone())),
            link: Some(link.clone()),
            title: Some(link.title),
            state: PartState::Link,
        }
    }
}

impl ListTrait for Part {
    /// method for creating wrapped text and making it a ListItem
    fn to_list_item(&self, width: u16) -> ListItem<'static> {
        let width = width.saturating_sub(4) as usize;

        match self.state {
            PartState::Text => {
                let mut lines = vec![];

                if let Some(title) = &self.title {
                    lines.push(Line::from(Span::raw(title.clone()).bold()));
                }

                if let Some(content) = &self.content {
                    parse_text(&mut lines, content.text.to_string(), width);
                }

                ListItem::new(Text::from(lines))
            }

            PartState::Link => {
                let link = self.link.clone().unwrap_or_default();
                let mut lines = vec![];

                // handle link objects
                if let Some(title) = &self.title {
                    lines.push(Line::from(Span::raw(title.clone()).bold()));
                }

                if let Some(text) = &self.content {
                    parse_text(&mut lines, text.text.to_string(), width);
                }

                if !link.url.is_empty() {
                    lines.push(Line::from(
                        Span::raw(format!("-> {}", link.url)).italic().cyan(),
                    ));
                }

                lines.push(Line::from(""));
                ListItem::new(Text::from(lines))
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Content {
    pub text: String,
    pub linecount: usize,
    pub wordcount: usize,
}

impl Content {
    pub fn new(text: String) -> Self {
        Self {
            text,
            linecount: 0,
            wordcount: 0,
        }
    }
}


