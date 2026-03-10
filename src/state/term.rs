use std::fmt::Display;

use crate::state::{input::InputState, tab_state::TabState};

#[derive(Debug, Clone, Default)]
pub struct TermState {
    pub err_msg: String,
    pub is_err: bool,
    pub input_state: Option<InputState>,
    pub tab_state: TabState,
    pub mode: Mode,
    pub exit: bool,
    pub scroll_idx: i32,
    pub cols: u16,
    pub lines: u16,
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
