use std::default;

use ratatui::{
    layout::Alignment,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{ListItem, Paragraph},
};

use crate::client::parser::Link;

#[derive(Debug, Clone)]
/// This struct represents a Page part:
/// Link Block, Text Block...
pub struct Part {
    state: PartState,
    title: Option<String>,
    text: Option<String>,
    link: Option<Link>,
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
            text: None,
            link: None,
        }
    }
}

// let line = Line::from("Hello, world!");
// let line = Line::from(String::from("Hello, world!"));
//
// let line = Line::from(vec![
//     Span::styled("Hello", Style::new().blue()),
//     Span::raw(" world!"),
// ]);
// ```

impl From<&Part> for ListItem<'_> {
    fn from(value: &Part) -> Self {
        let text = match value.state {
            PartState::Text => {
                let mut lines = vec![];

                if let Some(title) = &value.title {
                    if !title.is_empty() {
                        lines.push(Line::from(Span::raw(title.clone()).bold()));
                    }
                }

                // Add text content (will wrap)
                if let Some(text) = &value.text {
                    if !text.is_empty() {
                        lines.push(Line::from(text.clone()));
                    }
                }

                // Add empty line for spacing
                lines.push(Line::from(""));

                Text::from(lines)
            }
            PartState::Link => {
                let link = value.link.clone().unwrap_or_default();
                let mut lines = vec![];

                // Add title if present
                if let Some(title) = &value.title {
                    if !title.is_empty() {
                        lines.push(Line::from(Span::raw(title.clone()).bold().blue()));
                    }
                }

                // Add link text
                if let Some(text) = &value.text {
                    if !text.is_empty() {
                        lines.push(Line::from(text.clone()));
                    }
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

        return ListItem::new(text);
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
            text: Some(text),
            ..Default::default()
        }
    }

    pub fn link(link: Link) -> Self {
        Self {
            text: Some(link.text.clone()),
            link: Some(link.clone()),
            title: Some(link.title),
            ..Default::default()
        }
    }
}
