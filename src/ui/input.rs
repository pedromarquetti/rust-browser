use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    helpers::{calc_height, popup_area},
    state::input::InputState,
};

pub struct Input;

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(&mut self, area: Rect, buf: &mut Buffer, state: &mut InputState) {
        self.render(area, buf, state);
    }
}

impl StatefulWidget for &mut Input {
    type State = InputState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let width = 80.min(area.width.saturating_sub(4));
        let height = calc_height(state.input.value(), width, area, false);
        let popup_area = popup_area(area, width, height);

        state.input_area = popup_area;

        let paragraph = Paragraph::new(format!(": {:}", state.input.value()))
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(state.input_type.to_string()));

        Clear.render(popup_area, buf);
        Widget::render(paragraph, popup_area, buf);
    }
}
