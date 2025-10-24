use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::state::State;
use crate::ui::Mode;
use crate::ui::err_term::ErrorTerm;
use crate::ui::tabs::TabWidget;

#[derive(Debug, Clone)]
/// Widget for handling the top bar
pub struct Top {}

impl Top {
    pub fn new() -> Self {
        Top {}
    }

    pub fn create(&mut self, area: Rect, buf: &mut Buffer, state: &mut State) -> Result<()> {
        self.render(area, buf, state);
        Ok(())
    }
}

impl StatefulWidget for &mut Top {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.mode {
            Mode::Insert => {
                Paragraph::new("insert mode")
                    .block(Block::bordered())
                    .render(area, buf);
            }
            Mode::Normal => {
                TabWidget::new().create(area, buf, state);
            }
        }
    }
}
