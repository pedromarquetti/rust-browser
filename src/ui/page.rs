use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::client::parser::ParsedPage;

#[derive(Debug)]
pub struct Page {
    pub is_loading: bool,
    pub content: Option<ParsedPage>,
}

impl Page {
    pub fn create(&mut self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }
}

impl Widget for &Page {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.is_loading {
            Paragraph::new("Loading...")
                .block(Block::bordered())
                .centered()
                .render(area, buf);
        } else {
            Paragraph::new(format!("{:#?}", self.content))
                .block(Block::bordered())
                .render(area, buf);
        }
    }
}
