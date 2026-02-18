use std::fmt::Display;

use anyhow::Result;
use ratatui::layout::Rect;
use tui_input::Input;

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub input: Input,
    pub input_type: InputType,
    pub input_area: Rect,
}

impl InputState {
    pub fn new(input_type: InputType) -> Self {
        Self {
            input_type,
            ..Default::default()
        }
    }

    pub fn value(&self) -> &str {
        self.input.value()
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
