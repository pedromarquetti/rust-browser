use std::fmt::Display;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
    helpers::{calc_height, popup_area},
    state::{
        ListTrait,
        term::{PopupData, PopupState},
    },
};

#[derive(Debug, Default)]
pub struct PopupTerm {
    pub idx: i32,
}

#[derive(Debug, Clone)]
pub enum TermType {
    Info(PopupData),
    Error(PopupData),
    Warn(PopupData),
}

impl TermType {
    pub fn get_data(&self) -> &PopupData {
        match self {
            Self::Info(d) | Self::Warn(d) | Self::Error(d) => return d,
        }
    }

    pub fn err(data: PopupData) -> Self {
        TermType::Error(data)
    }

    pub fn info(data: PopupData) -> Self {
        TermType::Info(data)
    }

    pub fn warn(data: PopupData) -> Self {
        TermType::Warn(data)
    }
}

impl Default for TermType {
    fn default() -> Self {
        Self::Info(PopupData::Text(String::new()))
    }
}

impl Display for TermType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermType::Info(_) => {
                write!(f, "info")
            }
            TermType::Error(_) => {
                write!(f, "error")
            }
            TermType::Warn(_) => {
                write!(f, "warn")
            }
        }
    }
}

impl PopupTerm {
    pub fn new(idx: i32) -> Self {
        Self { idx }
    }

    pub fn handle_data_render(
        &self,
        data: PopupData,
        area: Rect,
        buf: &mut Buffer,
        state: &mut PopupState,
        block: Block,
    ) {
        let width = 80.min(area.width.saturating_sub(4));
        match &data {
            PopupData::Text(d) => {
                let msg = format!("{d}\n\nPress ESC to close");
                let height = calc_height(&msg, width, area, false);
                let popup_area = popup_area(area, width, height);
                let paragraph = Paragraph::new(msg)
                    .scroll((self.idx as u16, 0))
                    .wrap(Wrap { trim: false })
                    .block(block);
                Clear.render(popup_area, buf);
                Widget::render(paragraph, popup_area, buf);
            }
            PopupData::Links(links) => {
                let s: String = links
                    .iter()
                    .map(|i| {
                        return format!("\nlabel: {}\nurl: {}", i.text, i.url);
                    })
                    .collect();
                let inner = block.inner(area);
                let height = calc_height(&s, width, inner, false);
                let popup_area = popup_area(inner, width, height);
                let items: Vec<ListItem> = links.iter().map(|i| i.to_list_item(width)).collect();

                let list = List::new(items.clone()).highlight_symbol(">");
                let title = Line::from(format!("{} items", list.len()))
                    .style(Style::default().fg(Color::DarkGray).italic());

                let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(popup_area);
                Clear.render(popup_area, buf);
                block.render(popup_area, buf);
                StatefulWidget::render(list, list_area, buf, &mut state.list_state);
            }
        }
    }
}

impl StatefulWidget for &mut PopupTerm {
    type State = PopupState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.popup_type.clone() {
            TermType::Error(data) => {
                let block = Block::bordered()
                    .title("Error")
                    .border_style(Style::default().fg(Color::Red).bg(Color::Black));
                self.handle_data_render(data, area, buf, state, block);
            }
            TermType::Info(data) => {
                let block = Block::bordered()
                    .title("Info")
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Black));
                self.handle_data_render(data, area, buf, state, block);
            }
            TermType::Warn(data) => {
                let block = Block::bordered()
                    .title("Warning")
                    .border_style(Style::default().fg(Color::Yellow).bg(Color::Black));
                self.handle_data_render(data, area, buf, state, block);
            }
        }
    }
}
