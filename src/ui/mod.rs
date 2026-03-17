use ::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{prelude::*, style::Stylize, widgets::Clear};
use reqwest::Url;
use std::{str::FromStr, time::Duration};
use tracing::{error, info};
use tui_input::backend::crossterm::EventHandler;

use anyhow::{Context, Result};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::{
    state::input::InputType,
    ui::{
        input::Input,
        popup_term::{PopupTerm, TermType},
        top::Top,
    },
};
use crate::{
    state::{State, TaskType, term::Mode},
    ui::page::Page,
};

mod input;
mod page;
pub mod popup_term;
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
                    state.create_popup(e.to_string(), TermType::Error)
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, state: &mut State) {
        frame.render_stateful_widget(self, frame.area(), state);

        state.term_state.cols = frame.area().width;
        state.term_state.lines = frame.area().height;

        if state.term_state.mode == Mode::Insert
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
        if state.term_state.pop_up.is_some() {
            // do not allow any input other than Esc if err is active
            match e.code {
                KeyCode::Esc => {
                    state.close_popup();
                }
                _ => {
                    return Ok(());
                }
            }
        }
        match (e.code, state.term_state.mode.clone()) {
            (KeyCode::Esc, _) => {
                if state.term_state.pop_up.is_some() {
                    state.close_popup();
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
            (KeyCode::Down, Mode::Normal) => {}
            (KeyCode::Up, Mode::Normal) => {}
            (KeyCode::Char('p'), Mode::Normal) => state.term_state.tab_state.prev_tab()?,
            (KeyCode::Char('d'), Mode::Normal) => state.term_state.tab_state.del_tab()?,
            (KeyCode::Char('o'), Mode::Normal) => {
                // current selected item by cursor
                if let Ok(item) = state.term_state.tab_state.get_selected_item() {
                    if item.link.is_some() {
                        let url = Url::from_str(&item.link.unwrap_or_default().url)?;
                        state.go_to_url(url)?;
                    }
                }
            }
            // open in default browser
            (KeyCode::Enter, Mode::Normal) => {
                if let Ok(curr_item) = state.term_state.tab_state.get_selected_item() {
                    if curr_item.link.is_some() {
                        let url = curr_item.link.unwrap_or_default().url;
                        open::that_detached(&url)?;
                        state.create_popup(
                            format!("{} opened in default app!", url),
                            TermType::Info,
                        );
                    }
                    // early return to prevent double that_detached runs
                    return Ok(());
                }
                // because this runs if there's an active tab with content
                if let Some(tab) = state.term_state.tab_state.curr_tab() {
                    if let Some(content) = &tab.content {
                        open::that_detached(content.url.clone())?;
                        state.create_popup(
                            format!("{} opened in default app!", content.url),
                            TermType::Info,
                        );
                        return Ok(());
                    }
                }
            }
            (KeyCode::Enter, Mode::Insert) => {
                // TODO: maybe make a cache file with search history?
                if let Some(input_state) = state.term_state.input_state.take() {
                    let val = input_state.input.value().to_string();
                    if val.is_empty() || val == " " || val.split_whitespace().next().is_none() {
                        state.create_popup("No empty string allowed", TermType::Error);
                        return Ok(());
                    };

                    match input_state.input_type {
                        InputType::WebSearch => {
                            info!("User requested Web Search/URL request with {val}");
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
                                        info!(
                                            "{val} is a valid url, created tab {tab_id}, spawning page"
                                        );
                                        state.spawn_page(task_type, tab_id)?;
                                    }
                                }
                                Err(_) => {
                                    // input is valid but not URL
                                    info!(
                                        "searching for {val} using {:?}",
                                        state.web_client_state.search_provider
                                    );
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
                        InputType::StringSearch => {
                            info!("Entering search mode");
                            if let Some(tab) = state.term_state.tab_state.curr_tab_mut() {
                                if let Some(page) = tab.content.as_mut() {
                                    // resetting idx
                                    page.curr_search_idx = 0;
                                    page.get_search_pos(&val);
                                    if !page.pos.is_empty() {
                                        tab.scroll_idx = page.pos[0].line as u16;
                                    } else {
                                        error!("pattern {val} not found in search");
                                        state.create_popup(
                                            format!("Pattern {} not found!", val),
                                            TermType::Error,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    state.term_state.mode = Mode::Normal;
                }
            }
            (KeyCode::Char('t'), Mode::Normal) => state.next_search()?,
            (KeyCode::Char('T'), Mode::Normal) => state.prev_search()?,
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
                state.create_popup(err.to_string(), TermType::Error);
            }
        };

        Clear.render(page[0], buf);
        Block::default().bg(Color::Reset).render(page[0], buf);

        if let Some(tab) = state.term_state.tab_state.curr_tab_mut() {
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
            t.push_line("t/T -> next_prev search");
            t.push_line("d -> delete tab");
            t.push_line("q -> Quit App");
            t.push_line("Enter -> Open link in default OS browser");
            t.push_line("o -> Open link in this app");

            Paragraph::new(t)
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::new()
                        .title_bottom(state.term_state.mode.to_string())
                        .borders(Borders::all()),
                )
                .render(page[0], buf);
        }

        if state.term_state.mode == Mode::Insert
            && let Some(inputstate) = state.term_state.input_state.as_mut()
        {
            Input::new().create(area, buf, inputstate);
        }

        if let Some(data) = state.term_state.pop_up.as_ref() {
            PopupTerm::new(
                &data.popup_msg,
                state.term_state.scroll_idx,
                data.popup_type,
            )
            .render(area, buf);
        }
    }
}
