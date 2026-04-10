use std::fmt::Display;

use ratatui::widgets::ListState;

use crate::{
    client::parser::Link,
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
    pub popup_type: TermType,
    pub list_state: ListState,
}

#[derive(Debug, Clone)]
pub enum PopupData {
    Text(String),
    Links(Vec<Link>),
}

impl Display for PopupData {
    /// default to printing entire data
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Text(t) => write!(f, "{t}"),
            Self::Links(link) => write!(f, "{:?}", link),
        }
    }
}

impl Default for PopupData {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

impl PopupState {
    pub fn new(popup_type: TermType) -> Self {
        let mut state = ListState::default();
        // the list was jumping ramdomly (?)
        state.select(Some(0));
        Self {
            popup_type,
            list_state: state,
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
