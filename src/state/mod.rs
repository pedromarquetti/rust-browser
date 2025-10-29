use std::fmt::Display;

<<<<<<< HEAD
use crate::state::{
    input::InputState,
    tab_state::TabState,

pub mod input;
pub mod tab_state;
pub mod term;
=======
use crate::state::{tab_state::TabState, webclient_state::WebClientState};

pub mod tab_state;
pub mod webclient_state;
>>>>>>> http-client

#[derive(Debug, Default, Clone)]
/// Main App State
pub struct State {
<<<<<<< HEAD
    pub term_state: TermState,
=======
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
>>>>>>> http-client
}

impl State {
    pub fn close_app(mut self) {
        self.term_state.exit = true
    }

    /// main basic input field creator
    pub fn new_input(&mut self) {
        self.term_state.mode = Mode::Insert;
        self.term_state.input_state = Some(InputState::new())
    }

    pub fn cancel_input(&mut self) {
        self.term_state.mode = Mode::Normal;
        self.term_state.input_state = None
    }

    pub fn return_input(&mut self) -> Option<String> {
        self.term_state.mode = Mode::Normal;
        self.term_state.input_state = None;
        match &self.term_state.input_state {
            Some(input) => return Some(input.value.clone()),
            None => todo!(),
        }
    }
}
