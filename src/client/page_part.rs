use std::vec;

use ratatui::{
    style::Stylize,
    text::{Line, Span, Text},
    widgets::ListItem,
};

use crate::client::parser::Link;

#[derive(Debug, Clone)]
/// This struct represents a Page part:
/// Link Block, Text Block...
pub struct Part {
    state: PartState,
    pub title: Option<String>,
    pub text: Option<String>,
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
            text: None,
            link: None,
        }
    }
}

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
            state: PartState::Link,
        }
    }

    /// Helper method for wrapping String (used for direct url)
    pub fn to_wrapped_string( string: &mut String, width: u16) {
        let max = width.saturating_sub(4) as usize;
        if max == 0 || string.is_empty() {
            return;
        }

        let mut wrapped = String::new();
        for (i, line) in string.lines().enumerate() {
            // Preserve empty lines (paragraph breaks)
            if line.trim().is_empty() {
                if i != 0 {
                    wrapped.push('\n');
                }
                continue;
            }

            let mut curr = String::new();

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

        *string = wrapped;
    }

    /// method for creating wrapped text and making it a ListItem
    pub fn to_list_item(&self, width: u16) -> ListItem<'static> {
        let width = width.saturating_sub(4) as usize;

        match self.state {
            PartState::Text => {
                let mut lines = vec![];

                if let Some(title) = &self.title {
                    lines.push(Line::from(Span::raw(title.clone()).bold()));
                }

                if let Some(text) = &self.text {
                    // BUG: lines are not wrapping
                    parse_text(&mut lines, text.to_string(), width);
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

                if let Some(text) = &self.text {
                    parse_text(&mut lines, text.to_string(), width);
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

/// Helper function for handling text rendering (Vec<Line> line wrappin)
fn parse_text<'l>(lines: &mut Vec<Line<'l>>, text: String, width: usize) {
    if !text.is_empty() {
        for line in text.lines() {
            if line.len() <= width {
                lines.push(Line::from(line.to_string()));
            } else {
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut curr_line = String::new();

                for word in words {
                    if curr_line.len() + word.len() + 1 <= width {
                        if !curr_line.is_empty() {
                            curr_line.push(' ');
                        }
                        curr_line.push_str(word);
                    } else {
                        if !curr_line.is_empty() {
                            lines.push(Line::from(curr_line.clone()));
                        }
                        curr_line = word.to_string();
                    }
                }
                if !curr_line.is_empty() {
                    lines.push(Line::from(curr_line));
                }
            }
        }
    }

    lines.push(Line::from(""));
}
