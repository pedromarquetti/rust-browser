use anyhow::Result;
use ratatui::prelude::*;

use crate::state::State;
use crate::state::term::Mode;
use crate::ui::popup_term::TermType;
use crate::ui::tabs::TabWidget;

#[derive(Debug, Clone)]
/// Widget for handling the top bar
pub struct Top;

impl Default for Top {
    fn default() -> Self {
        Self::new()
    }
}

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
        if state.term_state.mode == Mode::Normal {
            match TabWidget::new().create(area, buf, state) {
                Ok(ok) => ok,
                Err(err) => {
                    state.create_popup(err.to_string(), TermType::Error);
                }
            }
        }
    }
}
