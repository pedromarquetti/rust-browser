use crate::client::parser::{PageType, ParsedContent, StrPos};
use crate::state::State;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget, Wrap};

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

            let scroll_idx: u16 = tab.scroll_idx;

            let title = Line::from(content.title.clone()).style(Style::default());
            tab.title = content.title.clone();

            let block = Block::default()
                .borders(Borders::all())
                .title(title)
                .title_bottom(state.term_state.mode.to_string())
                .bg(Color::Reset);

            let inner = block.inner(area);
            let available_width = inner.width;

            Clear.render(inner, buf);
            match content.page_type {
                PageType::Search => match &content.parsed_content {
                    ParsedContent::PartList(list) => {
                        let items: Vec<ListItem> = list
                            .iter()
                            .map(|part| part.to_list_item(available_width))
                            .collect();
                        let list = List::new(items.clone()).highlight_symbol(">");
                        let title = Line::from(format!("{} items", list.len()))
                            .style(Style::default().fg(Color::DarkGray).italic());

                        let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(inner);
                        block.title(title).render(area, buf);
                        Clear.render(inner, buf);
                        StatefulWidget::render(list, list_area, buf, &mut content.state);
                    }
                    _ => {}
                },
                PageType::Raw => {
                    Clear.render(inner, buf);
                    // wrapping
                    content.to_wrapped_string(state.term_state.cols);
                    let wordcount =
                        format!("words: {} lines: {}", content.wordcount, content.linecount);
                    let pos: &StrPos = {
                        match content.pos.get(content.curr_search_idx as usize) {
                            Some(i) => i,
                            None => &StrPos {
                                ..Default::default()
                            },
                        }
                    };

                    let details =
                        Line::from(wordcount).style(Style::default().fg(Color::DarkGray).italic());
                    match &content.parsed_content {
                        ParsedContent::Text(text) => {
                            block
                                .title(details)
                                .title_bottom(pos.to_string())
                                .render(area, buf);
                            Clear.render(inner, buf);
                            Paragraph::new(text.clone())
                                .scroll((scroll_idx as u16, 0))
                                .wrap(Wrap { trim: false })
                                .render(inner, buf);
                        }
                        _ => {}
                    }
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
