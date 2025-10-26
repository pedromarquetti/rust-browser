use std::fmt::Display;

use crate::state::{input::InputState, tabs::TabState};

#[derive(Debug, Clone, Default)]
pub struct TermState {
    pub is_err: bool,
    pub input_state: Option<InputState>,
    pub tab_state: TabState,
    pub mode: Mode,
    pub exit: bool,
    pub curr_key: String,
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
