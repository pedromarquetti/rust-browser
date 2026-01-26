use std::fmt::Display;

use anyhow::Result;
use ratatui::layout::Rect;

use crate::state::cursor::Cursor;

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub value: String,
    pub input_type: InputType,
    pub cursor: Cursor,
    pub input_area: Rect,
}

impl InputState {
    pub fn new(input_type:InputType) -> Self {
        Self {
            input_type,
            ..Default::default()
        }
    }

    pub fn set_cursor_pos(&mut self, posx: Option<usize>, posy: Option<usize>) {
        if let Some(x) = posx {
            self.cursor.set_posx(x);
        }
        if let Some(y) = posy {
            self.cursor.set_posy(y);
        }
    }

    pub fn insert_char(&mut self, c: char, max_cols: usize) {
        self.value.insert(self.cursor.get_pos().0, c);
        self.cursor.move_right(max_cols);
    }

    pub fn backspace(&mut self) -> Result<()> {
        if self.value == "" {
            return Ok(());
        }

        let prev = self.value[..self.cursor.get_pos().0]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.value.drain(prev..self.cursor.get_pos().0);
        self.cursor.move_left();

        Ok(())
    }

    pub fn delete(&mut self) {
        if self.cursor.get_pos().0 >= self.value.len() {
            return;
        }
        let next = self.value[self.cursor.get_pos().0..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor.get_pos().0 + i)
            .unwrap_or(self.value.len());
        self.value.drain(self.cursor.get_pos().0..next);
    }

    pub fn move_left(&mut self) {
        if self.cursor.get_pos().0 == 0 {
            return;
        }
        self.cursor.move_left();
    }

    pub fn move_right(&mut self, max_cols: usize) {
        if self.cursor.get_pos().0 >= self.value.len() {
            return;
        }
        self.cursor.move_right(max_cols);
    }

    pub fn move_home(&mut self) {
        self.cursor.move_home();
    }

    pub fn move_end(&mut self) {
        self.cursor.move_end(self.value.len());
    }
}

#[derive(Debug, Default, Clone)]
pub enum InputType {
    #[default]
    WebSearch,
    StringSearch,
}

impl Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::WebSearch => {
                write!(f, "web search")
            }
            Self::StringSearch => {
                write!(f, "string search")
            }
        }
    }
}
