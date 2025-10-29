
use crate::state::{input::InputState, term::{Mode, TermState}, webclient_state::WebClientState};

pub mod input;
pub mod tab_state;
pub mod term;
pub mod webclient_state;

#[derive(Debug, Default, Clone)]
/// Main App State
pub struct State {
    pub term_state: TermState,
    pub web_client_state: WebClientState,
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
