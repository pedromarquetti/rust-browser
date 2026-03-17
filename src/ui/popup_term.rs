use std::fmt::Display;

use anyhow::Result;
use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::helpers::{calc_height, popup_area};

#[derive(Debug, Default)]
pub struct PopupTerm<'a> {
    pub msg: &'a str,
    pub idx: i32,
    pub term_type: TermType,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TermType {
    #[default]
    Info,
    Error,
    Warn,
}

impl Display for TermType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermType::Info => {
                write!(f, "info")
            }
            TermType::Error => {
                write!(f, "error")
            }
            TermType::Warn => {
                write!(f, "warn")
            }
        }
    }
}

impl<'a> PopupTerm<'a> {
    pub fn new(msg: &'a str, idx: i32, term_type: TermType) -> Self {
        Self {
            msg,
            idx,
            term_type,
        }
    }

    pub fn create(&self, area: Rect, buf: &mut Buffer) -> Result<()> {
        self.render(area, buf);
        Ok(())
    }
}

impl<'a> Widget for &PopupTerm<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let width = 80.min(area.width.saturating_sub(4));
        let height = calc_height(self.msg, width, area, false);

        let popup_area = popup_area(area, width, height);

        let mut msg = self.msg.to_string();
        msg.push_str("\n\nPress Esc to close!");

        match self.term_type {
            TermType::Error => {
                let paragraph = Paragraph::new(msg)
                    .scroll((self.idx as u16, 0))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::bordered()
                            .title("Error")
                            .border_style(Style::default().fg(Color::Red).bg(Color::Black)),
                    );
                Clear.render(popup_area, buf);
                Widget::render(paragraph, popup_area, buf);
            }
            TermType::Info => {
                let paragraph = Paragraph::new(msg)
                    .scroll((self.idx as u16, 0))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::bordered()
                            .title("Info")
                            .border_style(Style::default().fg(Color::Blue).bg(Color::Black)),
                    );
                Clear.render(popup_area, buf);
                Widget::render(paragraph, popup_area, buf);
            }
            TermType::Warn => {
                let paragraph = Paragraph::new(msg)
                    .scroll((self.idx as u16, 0))
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::bordered()
                            .title("Warning")
                            .border_style(Style::default().fg(Color::Yellow).bg(Color::Black)),
                    );
                Clear.render(popup_area, buf);
                Widget::render(paragraph, popup_area, buf);
            }
        }
    }
}
