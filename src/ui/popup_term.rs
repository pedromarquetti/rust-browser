use std::fmt::Display;

use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::{
    helpers::{calc_height, popup_area},
    state::term::{PopupData, PopupState},
};

#[derive(Debug, Default)]
pub struct PopupTerm {
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

impl PopupTerm {
    pub fn new(idx: i32, term_type: TermType) -> Self {
        Self { idx, term_type }
    }
}

impl StatefulWidget for &mut PopupTerm {
    type State = PopupState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let width = 80.min(area.width.saturating_sub(4));
        let height = calc_height(&state.data.to_string(), width, area, false);

        let popup_area = popup_area(area, width, height);

        let mut msg = state.data.to_string();
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

// impl Widget for &mut PopupTerm {
//     fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
//     where
//         Self: Sized,
//     {
//         let width = 80.min(area.width.saturating_sub(4));
//         // let height = calc_height(self.data, width, area, false);
//         let height = calc_height("", width, area, false);
//
//         let popup_area = popup_area(area, width, height);
//
//         // let mut msg = self.data.to_string();
//         // msg.push_str("\n\nPress Esc to close!");
//
//         // TODO: the infobox should render a list if one is supplied
//         // ListState for popup needs to be implemented
//
//         // match self.term_type {
//         //     TermType::Error => {
//         //         let paragraph = Paragraph::new(msg)
//         //             .scroll((self.idx as u16, 0))
//         //             .wrap(Wrap { trim: false })
//         //             .block(
//         //                 Block::bordered()
//         //                     .title("Error")
//         //                     .border_style(Style::default().fg(Color::Red).bg(Color::Black)),
//         //             );
//         //         Clear.render(popup_area, buf);
//         //         Widget::render(paragraph, popup_area, buf);
//         //     }
//         //     TermType::Info => {
//         //         let paragraph = Paragraph::new(msg)
//         //             .scroll((self.idx as u16, 0))
//         //             .wrap(Wrap { trim: false })
//         //             .block(
//         //                 Block::bordered()
//         //                     .title("Info")
//         //                     .border_style(Style::default().fg(Color::Blue).bg(Color::Black)),
//         //             );
//         //         Clear.render(popup_area, buf);
//         //         Widget::render(paragraph, popup_area, buf);
//         //     }
//         //     TermType::Warn => {
//         //         let paragraph = Paragraph::new(msg)
//         //             .scroll((self.idx as u16, 0))
//         //             .wrap(Wrap { trim: false })
//         //             .block(
//         //                 Block::bordered()
//         //                     .title("Warning")
//         //                     .border_style(Style::default().fg(Color::Yellow).bg(Color::Black)),
//         //             );
//         //         Clear.render(popup_area, buf);
//         //         Widget::render(paragraph, popup_area, buf);
//         //     }
//         // }
//     }
// }
