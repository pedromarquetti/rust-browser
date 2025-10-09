use std::fmt::Display;

use crate::state::tabs::TabState;

pub mod tabs;

#[derive(Debug, Default, Clone)]
/// Main App State
pub struct State {
    pub is_err: bool,
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

impl State {
    pub fn close_app(mut self) {
        self.exit = true
    }
}
