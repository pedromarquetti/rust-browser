use anyhow::Result;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::helpers::{calc_height, popup_area};

#[derive(Debug, Default)]
pub struct ErrorTerm<'a> {
    pub msg: &'a str,
    pub idx: i32,
}

impl<'a> ErrorTerm<'a> {
    pub fn new(msg: &'a str, idx: i32) -> Self {
        Self { msg, idx }
    }

    pub fn create(&self, area: Rect, buf: &mut Buffer) -> Result<()> {
        self.render(area, buf);
        Ok(())
    }
}

impl<'a> Widget for &ErrorTerm<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let width = 80.min(area.width.saturating_sub(4));
        let height = calc_height(self.msg, width, area, false);

        let popup_area = popup_area(area, width, height);

        let mut msg = self.msg.to_string();
        msg.push_str("\n\nPress Esc to exit!");

        let paragraph = Paragraph::new(msg)
            .scroll((self.idx as u16, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title("Error")
                    .border_style(Style::default().fg(Color::Red).bg(Color::Black)),
            );

        Clear.render(popup_area, buf);
        Widget::render(paragraph, popup_area, buf);
    }
}
