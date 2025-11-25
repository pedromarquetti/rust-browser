use anyhow::Result;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
};

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

    fn calc_height(&self, width: u16, area: Rect) -> u16 {
        let available_width = width.saturating_sub(4) as usize; // Account for borders and padding
        let mut total_lines = 0;

        for line in self.msg.lines() {
            if line.is_empty() {
                total_lines += 1;
            } else {
                // Calculate wrapped lines for this text line
                let chars = line.chars().count();
                let wrapped_lines = (chars / available_width).max(1);
                total_lines += wrapped_lines;
            }
        }

        // Add lines for footer message
        total_lines += 2; // "Press ESC to clear error" + empty line

        // Add padding for borders
        (total_lines as u16)
            .saturating_add(4)
            .min(area.height.saturating_sub(4))
    }
}

impl<'a> Widget for &ErrorTerm<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let width = 80.min(area.width.saturating_sub(4));
        let height = self.calc_height(width, area);

        let popup_area = popup_area(area, width, height);

        let mut msg = self.msg.to_string();
        msg.push_str("\n\nPress Esc to exit!");

        let paragraph = Paragraph::new(msg)
            .scroll((self.idx as u16, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title("Error")
                    .border_style(Style::default().fg(Color::Red)),
            );

        Widget::render(paragraph, popup_area, buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);

    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);

    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
