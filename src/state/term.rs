use std::fmt::Display;

use crate::{
    state::{input::InputState, tab_state::TabState},
    ui::popup_term::TermType,
};

#[derive(Debug, Clone, Default)]
pub struct TermState {
    pub pop_up: Option<PopupState>,
    pub input_state: Option<InputState>,
    pub tab_state: TabState,
    pub mode: Mode,
    pub exit: bool,
    pub scroll_idx: i32,
    pub cols: u16,
    pub lines: u16,
}

#[derive(Debug, Clone, Default)]
pub struct PopupState {
    // pop-up configs
    pub popup_msg: String,
    pub popup_type: TermType,
}

impl PopupState {
    pub fn new<S: Into<String>>(popup_type: TermType, popup_msg: S) -> Self {
        Self {
            popup_msg: popup_msg.into(),
            popup_type,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Mode {
    Insert,
    #[default]
    Normal,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "({}, {})", self., self.latitude)
        match &self {
            Self::Insert => {
                write!(f, "insert")
            }
            Self::Normal => {
                write!(f, "normal")
            }
        }
    }
}
