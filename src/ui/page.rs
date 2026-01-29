use std::clone;

use crate::client::page_part::Part;
use crate::client::parser::{PageType, ParsedContent};
use crate::state::State;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};

#[derive(Debug, Default)]
pub struct Page {
    pub is_loading: bool,
}

impl Page {
    pub fn create(&mut self, area: Rect, buf: &mut Buffer, state: &mut State) {
        self.render(area, buf, state);
    }
}

impl StatefulWidget for &mut Page {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let tab = match state.term_state.tab_state.curr_tab.as_mut() {
            Some(tab) => tab,
            None => {
                return;
            }
        };

        if !tab.is_loading {
            let content = match tab.content.as_mut() {
                Some(content) => content,
                None => {
                    return;
                }
            };

            let title = Line::from(content.title.clone()).style(Style::default());
            tab.title = content.title.clone();
            let wordcount = format!("words: {} lines: {}", tab.wordcount, tab.linecount);
            let details = Line::from(wordcount).style(Style::default().fg(Color::DarkGray));

            let block = Block::default()
                .borders(Borders::all())
                .title(title)
                .title(details)
                .bg(Color::Reset);

            let inner = block.inner(area);
            let available_width = inner.width;
            let scroll_idx: i32 = tab.scroll_idx;

            block.render(area, buf);

            Clear.render(inner, buf);

            match content.page_type {
                PageType::Search => {
                    match &content.parsed_content {
                        ParsedContent::PartList(list) => {
                            let items: Vec<ListItem> = list
                                .iter()
                                .map(|part| part.to_list_item(available_width))
                                .collect();
                            let list = List::new(items.clone()).highlight_symbol(">");

                            let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(inner);
                            Clear.render(inner, buf);
                            StatefulWidget::render(list, list_area, buf, &mut content.state);
                        }
                        _ => {}
                    }

                }
                PageType::Raw => {
                    Clear.render(inner, buf);
                    Paragraph::new("oi").render(inner, buf);
                }
            }

        } else {
            Clear.render(area, buf);
            Paragraph::new("Loading...")
                .centered()
                .block(Block::new().borders(Borders::all()))
                .render(area, buf);
        }
    }
}
