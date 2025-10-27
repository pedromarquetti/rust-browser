use std::mem::take;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::state::{Mode, State};
use crate::ui::{err_term::ErrorTerm, top::Top};

mod err_term;
mod page;
mod tabs;
mod top;

#[derive(Debug)]
pub struct Term {}

impl Term {
    pub fn new() -> Term {
        Term {}
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, state: &mut State) -> Result<()> {
        // while !self.exit {
        while !state.exit {
            terminal
                .draw(|frame| self.draw(frame, state))
                .context("Failed to run terminal.draw!")?;
            self.handle_event(state)
                .context("Failed to handle event!")?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, state: &mut State) {
        frame.render_stateful_widget(self, frame.area(), state);
        if state.term_state.mode == Mode::Insert {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
                .split(frame.area());

            let top = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Min(10)])
                .split(main_layout[0]);

            if let Some(input) = state.term_state.input_state.as_ref() {
                let prefix = " ";
                let prefix_len = prefix.len() as u16;
                let cursor_cols = input.value[..input.cursor].chars().count() as u16;
                let x = top[0].x + 1 + prefix_len + cursor_cols; // +1 to be inside the bordered block
                let y = top[0].y + 1;
                frame.set_cursor_position(Position::new(x, y));
            }
        }
    }

    /// main event handler
    pub fn handle_event(&mut self, state: &mut State) -> Result<()> {
        match event::read()? {
            // handles only key press
            Event::Key(event) if event.kind == KeyEventKind::Press => {
                self.handle_keypress(event, state)?
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_keypress(&mut self, e: KeyEvent, state: &mut State) -> Result<()> {
        match (e.code, state.term_state.mode.clone()) {
            (KeyCode::Esc, _) => state.term_state.mode = Mode::Normal,
            (KeyCode::Char('q'), Mode::Normal) => state.term_state.exit = true,
            (KeyCode::Char('i'), Mode::Normal) | (KeyCode::Char('s'), Mode::Normal) => {
                state.new_input();
            }
            (KeyCode::Char('n'), Mode::Normal) => state.term_state.tab_state.next_tab()?,
            (KeyCode::Char('p'), Mode::Normal) => state.term_state.tab_state.prev_tab()?,
            (KeyCode::Char('d'), Mode::Normal) => state.term_state.tab_state.del_tab(),
            (KeyCode::Enter, Mode::Insert) => {
                if let Some(mut val) = state.term_state.input_state.take() {
                    let val = take(&mut val.value);
                    state.term_state.mode = Mode::Normal;
                    state.term_state.tab_state.new_tab(val);
                }
            }
            (KeyCode::Backspace, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    if input.cursor > 0 {
                        let prev = input.value[..input.cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        input.value.drain(prev..input.cursor);
                        input.cursor = prev;
                    }
                }
            }
            (KeyCode::Delete, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    if input.cursor < input.value.len() {
                        let next = input.value[input.cursor..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| input.cursor + i)
                            .unwrap_or(input.value.len());
                        input.value.drain(input.cursor..next);
                    }
                }
            }
            (KeyCode::Left, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    if input.cursor > 0 {
                        input.cursor = input.value[..input.cursor]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                    }
                }
            }
            (KeyCode::Right, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    if input.cursor < input.value.len() {
                        input.cursor = input.value[input.cursor..]
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| input.cursor + i)
                            .unwrap_or(input.value.len());
                    }
                }
            }
            (KeyCode::Home, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    input.cursor = 0;
                }
            }
            (KeyCode::End, Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    input.cursor = input.value.len();
                }
            }

            // insert text
            (KeyCode::Char(c), Mode::Insert) => {
                if let Some(input) = state.term_state.input_state.as_mut() {
                    input.value.insert(input.cursor, c);
                    input.cursor += c.len_utf8();
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl StatefulWidget for &mut Term {
    type State = State;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(10)])
            .split(main_layout[0]);

        // main content
        let page = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100)])
            .split(main_layout[1]);

        match Top::new()
            .create(top[0], buf, state)
            .context("Error while creating Top widget")
        {
            Ok(ok) => {ok}
            Err(err) => {
                ErrorTerm::new(err.to_string()).render(area, buf);
            }
        };

        if state.term_state.tab_state.tab_list.len() == 0 && state.mode == Mode::Normal {
            Paragraph::new(
                "Welcome to my simple Terminal Broswer".to_string()
                    + "\n\n"
                    + "i -> insert mode\n"
                    + "Esc -> Normal mode\n"
                    + "In normal mode: \t\n"
                    + "t -> New Tab\t\n"
                    + "n -> next tab\t\n"
                    + "p -> prev. tab\t\n"
                    + "d -> delete tab\t\n",
            )
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::new().borders(Borders::all()))
            .render(area, buf);
        }

        let curr_tab: Tab = state.term_state.tab_state.curr_tab.clone();

        Paragraph::new(format!(
            "idx{},\ntitle:{} id{} ",
            state.term_state.tab_state.idx, curr_tab.title, curr_tab.id
        ))
        .block(Block::bordered())
        .render(page[0], buf);
    }
}
