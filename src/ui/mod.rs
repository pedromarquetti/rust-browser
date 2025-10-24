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
    }

    /// main event handler
    pub fn handle_event(&mut self, state: &mut State) -> Result<()> {
        match event::read()? {
            // handles only key press
            Event::Key(event) if event.kind == KeyEventKind::Press => {
                self.handle_keypress(event, state)
            }
            _ => {}
        }
        Ok(())
    }

    pub fn handle_keypress(&mut self, e: KeyEvent, state: &mut State) {
        match (e.code, state.mode.clone()) {
            (KeyCode::Esc, _) => state.mode = Mode::Normal,
            (KeyCode::Char('q'), Mode::Normal) => state.exit = true,
            (KeyCode::Char('i'), _) => state.mode = Mode::Insert,
            (KeyCode::Char('n'), Mode::Normal) => state.tab_state.next_tab(),
            (KeyCode::Char('p'), Mode::Normal) => state.tab_state.prev_tab(),
            (KeyCode::Char('t'), Mode::Normal) => state.tab_state.new_tab(),
            (KeyCode::Char('d'), Mode::Normal) => state.tab_state.del_tab(),
            _ => {}
        }
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
            Ok(()) => {}
            Err(err) => {
                ErrorTerm::new(err.to_string()).render(area, buf);
            }
        };

        if state.tab_state.tab_list.len() == 0 && state.mode == Mode::Normal {
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
    }
}
