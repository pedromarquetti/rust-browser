use anyhow::Result;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};

#[derive(Debug, Default)]
pub struct ErrorTerm {
    pub msg: String,
}

impl ErrorTerm {
    pub fn new(msg: String) -> Self {
        Self { msg: msg }
    }

    pub fn create(&self, area: Rect, buf: &mut Buffer) -> Result<()> {
        self.render(area, buf);
        Ok(())
    }
}

impl Widget for &ErrorTerm {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // let block = Block::bordered().title("Error");
        let block = Paragraph::new(self.msg.clone()).block(Block::bordered().title("Error"));

        let area = popup_area(area, 60, 20);

        // Widget::render(Clear, area, buf);
        Widget::render(block, area, buf);
        Paragraph::new(self.msg.clone())
            .block(Block::bordered().title("Error"))
            .render(area, buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
