use std::fmt::Display;

use crate::state::{tab_state::TabState, webclient_state::WebClientState};

pub mod tab_state;
pub mod webclient_state;

#[derive(Debug, Default, Clone)]
/// Main App State
pub struct State {
    pub is_err: bool,
    pub tab_state: TabState,
    pub web_client_state: WebClientState,
    pub mode: Mode,
    pub exit: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Mode {
    Insert,
    #[default]
    Normal,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
