use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::state::State;
use crate::state::term::Mode;
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
        match state.term_state.mode {
            Mode::Insert => {
                let val = state
                    .term_state
                    .input_state
                    .as_ref()
                    .map(|i| i.value.as_str())
                    .unwrap_or("");
                Paragraph::new(format!(":{}", val))
                    .block(Block::bordered())
                    .render(area, buf);
            }
            Mode::Normal => match TabWidget::new().create(area, buf, state) {
                Ok(ok) => ok,
                Err(err) => {
                    todo!("implement error handling")
                }
            },
        }
    }
}
