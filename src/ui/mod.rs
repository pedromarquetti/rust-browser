use ::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, style::Stylize, widgets::Clear};
use reqwest::Url;
use std::{str::FromStr, time::Duration};
use tui_input::backend::crossterm::EventHandler;

use anyhow::{Context, Result};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::{
    state::input::InputType,
    ui::{err_term::ErrorTerm, input::Input, top::Top},
};
use crate::{
    state::{State, TaskType, term::Mode},
    ui::page::Page,
};

mod err_term;
mod input;
mod page;
mod tabs;
mod top;

#[derive(Debug)]
pub struct Term {}

impl Default for Term {
    fn default() -> Self {
        Self::new()
    }
}

impl Term {
    pub fn new() -> Term {
        Term {}
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, state: &mut State) -> Result<()> {
        while !state.term_state.exit {
            state.process_task_results();
            terminal
                .draw(|frame| self.draw(frame, state))
                .context("Failed to run terminal.draw!")?;

            match self.handle_event(state) {
                Ok(_) => {}
                Err(e) => {
                    // dont't crash if an error was returned after pressing the
                    // wrong key
                    state.create_err(e.to_string());
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, state: &mut State) {
        frame.render_stateful_widget(self, frame.area(), state);

        state.term_state.cols = frame.area().width;

        if state.term_state.mode == Mode::Insert
            && state.term_state.tab_state.curr_tab.is_none()
            && let Some(input_state) = state.term_state.input_state.as_ref()
        {
            // derive screen cursor from input state
            let prefix_len: u16 = 2; // ": "
            let typed_len = input_state.input.visual_cursor() as u16;
            let x = input_state.input_area.x + 1 + prefix_len + typed_len;
            let y = input_state.input_area.y + 1;

            frame.set_cursor_position(Position::new(x, y));
        }
    }

    /// main event handler
    pub fn handle_event(&mut self, state: &mut State) -> Result<()> {
        if event::poll(Duration::from_nanos(100))? {
            match event::read()? {
                // handles only key press
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    self.handle_keypress(event, state)?
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn handle_keypress(&mut self, e: KeyEvent, state: &mut State) -> Result<()> {
        match (e.code, state.term_state.mode.clone()) {
            (KeyCode::Esc, _) => {
                if state.term_state.is_err {
                    state.term_state.is_err = false;
                    state.term_state.err_msg = String::new()
                }

                state.cancel_input();
            }
            (KeyCode::Char('q'), Mode::Normal) => state.term_state.exit = true,
            (KeyCode::Char('k'), Mode::Normal) => {
                state.handle_up()?;
            }
            (KeyCode::Char('j'), Mode::Normal) => {
                state.handle_down()?;
            }
            (KeyCode::Char('i'), Mode::Normal) | (KeyCode::Char('s'), Mode::Normal) => {
                state.new_input(InputType::WebSearch);
            }
            (KeyCode::Char('/'), Mode::Normal) => state.new_input(InputType::StringSearch),
            (KeyCode::Char('n'), Mode::Normal) => state.term_state.tab_state.next_tab()?,
            (KeyCode::Char('p'), Mode::Normal) => state.term_state.tab_state.prev_tab()?,
            (KeyCode::Char('d'), Mode::Normal) => state.term_state.tab_state.del_tab()?,
            (KeyCode::Char('o'), Mode::Normal) => {
                // current selected item by cursor
                let curr_item = state.term_state.tab_state.get_selected_item()?;

                if curr_item.link.is_some() {
                    let url = Url::from_str(&curr_item.link.unwrap_or_default().url)?;
                    state.go_to_url(url)?;
                }
            }
            // open in default browser
            (KeyCode::Enter, Mode::Normal) => {
                let curr_item = state.term_state.tab_state.get_selected_item()?;

                if curr_item.link.is_some() {
                    // TODO: make this open the link in a new tab
                    // currently this will open in the current browser
                    open::that_detached(curr_item.link.unwrap_or_default().url)?;
                }
            }
            (KeyCode::Enter, Mode::Insert) => {
                // TODO: maybe make a cache file with search history?
                if let Some(mut input_state) = state.term_state.input_state.take() {
                    // let val = take(&mut val.value);
                    let val = input_state.input.value().to_string();
                    if val.is_empty() || val == " " || val.split_whitespace().next().is_none() {
                        state.create_err("No empty string allowed");
                    } else {
                        match Url::from_str(&val) {
                            Ok(url) => {
                                let schema = url.scheme();
                                if schema.starts_with("https") || schema.starts_with("http") {
                                    let task_type = TaskType::Url(Url::from_str(&val)?);
                                    state.term_state.mode = Mode::Normal;
                                    let tab_id = state
                                        .term_state
                                        .tab_state
                                        .new_tab(val.clone(), task_type.clone())
                                        .context("Cannot create tab!")?;
                                    state.spawn_page(task_type, tab_id)?;
                                }
                            }
                            Err(_) => {
                                // input is valid but not URL
                                let task_type = TaskType::Search(val.clone());
                                state.term_state.mode = Mode::Normal;
                                let tab_id = state
                                    .term_state
                                    .tab_state
                                    .new_tab(val.clone(), task_type.clone())
                                    .context("Cannot create tab!")?;
                                state.spawn_page(task_type, tab_id)?;
                            }
                        }
                    }
                }
            }
            (_, Mode::Insert) => {
                if let Some(input_state) = state.term_state.input_state.as_mut() {
                    input_state.input.handle_event(&Event::Key(e));
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
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0)])
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
            Ok(ok) => ok,
            Err(err) => {
                state.create_err(err.to_string());
            }
        };

        Clear.render(page[0], buf);
        Block::default().bg(Color::Reset).render(page[0], buf);

        if let Some(tab) = &state.term_state.tab_state.curr_tab {
            let mut p = Page {
                is_loading: tab.is_loading,
            };
            p.create(page[0], buf, state);
        } else if state.term_state.input_state.is_none() {
            let mut t = Text::from("");
            t.push_line("Welcome to my simple Terminal Broswer".bold());
            t.push_line("Esc -> Normal mode");
            t.push_line("");
            t.push_line("In normal mode: ".bold());
            t.push_line("n -> next tab");
            t.push_line("p -> prev. tab");
            t.push_line("s -> search the web");
            t.push_line("j/k -> scroll");
            t.push_line("d -> delete tab");
            t.push_line("q -> Quit App");
            t.push_line("Enter -> Open link in default OS browser");
            t.push_line("o -> Open link in this app");

            Paragraph::new(t)
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::new().borders(Borders::all()))
                .render(page[0], buf);
        }

        if state.term_state.mode == Mode::Insert
            && let Some(inputstate) = state.term_state.input_state.as_mut()
        {
            Input::new().create(area, buf, inputstate);
        }

        if state.term_state.is_err {
            ErrorTerm::new(&state.term_state.err_msg, state.term_state.scroll_idx)
                .render(area, buf);
        }
    }
}
