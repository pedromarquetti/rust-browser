use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    helpers::{calc_height, popup_area},
    state::{input::InputState, term::TermState},
};

pub struct Input {
    text: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
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
        let height = calc_height(&state.value, width, area, false);
        let popup_area = popup_area(area, width, height);

        state.input_area = popup_area;

        let prefix = " ";
        let prefix_len = prefix.len() as u16;

        // ..input.cursor.get_pos().0 gets the last chat count in input
        let max_input_size = state.value[..state.cursor.get_pos().0].chars().count() as u16;

        let x = state.input_area.x + prefix_len + max_input_size;
        // +1 to be inside the bordered block
        let y = state.input_area.y;

        state.set_cursor_pos(x as usize, y as usize);

        let paragraph = Paragraph::new(format!("{:}", state.value))
            .wrap(Wrap { trim: false })
            .block(Block::bordered());

        Clear.render(popup_area, buf);
        Widget::render(paragraph, popup_area, buf);
    }
}
