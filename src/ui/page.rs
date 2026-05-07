use crate::client::parser::{PageType, ParsedContent, StrPos};
use crate::state::State;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, Paragraph, Widget};

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
        let tab = match state.term_state.tab_state.curr_tab_mut() {
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
                .title_bottom(state.term_state.mode.to_string())
                .bg(Color::Reset);

            let inner = block.inner(area);
            let available_width = inner.width;

            Clear.render(inner, buf);
            match content.page_type {
                PageType::Search => match &content.parsed_content {
                    ParsedContent::PartList(_) => {
                        // creating cache of ListItems (search page)
                        content.search_items_cache(available_width);
                        let items = content.search_items();
                        let list_len = Line::from(format!("{} items", content.search_items_len()))
                            .style(Style::default().fg(Color::DarkGray).italic());
                        let list = List::new(items).highlight_symbol(">");

                        let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(inner);
                        block.title(title).title(list_len).render(area, buf);
                        Clear.render(inner, buf);
                        StatefulWidget::render(
                            list,
                            list_area,
                            buf,
                            &mut content.state.borrow_mut(),
                        );
                    }
                    _ => {}
                },
                PageType::Raw => {
                    Clear.render(inner, buf);

                    // wrap parsed content 
                    // This is needed for string search functionallity
                    content.to_wrapped_string(available_width);

                    let wordcount = format!(
                        "words: {} lines: {}",
                        content.wordcount.get().unwrap_or_default(),
                        content.linecount.get().unwrap_or_default()
                    );

                    let pos: StrPos = {
                        let p = content.pos.borrow();
                        match p.get(content.curr_search_idx.get() as usize) {
                            Some(i) => i.clone(),
                            None => StrPos {
                                ..Default::default()
                            },
                        }
                    };

                    let details =
                        Line::from(wordcount).style(Style::default().fg(Color::DarkGray).italic());

                    // long page titles should be cut
                    let mut cut_title = title.to_string();
                    let limit = (available_width as usize) / 2;
                    let end_index = cut_title
                        .char_indices()
                        .nth(limit)
                        .map(|(idx, _)| idx)
                        .unwrap_or(cut_title.len());
                    cut_title.truncate(end_index);

                    block
                        .title(cut_title)
                        .title(details)
                        .title_bottom(pos.to_string())
                        .render(area, buf);

                    Clear.render(inner, buf);
                    match &content.parsed_content {
                        ParsedContent::Text(text) => {
                            Paragraph::new(text.clone())
                                .scroll((scroll_idx, 0))
                                .wrap(ratatui::widgets::Wrap { trim: false })
                                .render(inner, buf);
                        }
                        _ => {
                            Paragraph::new(content.raw_text.as_str())
                                .scroll((scroll_idx, 0))
                                .wrap(ratatui::widgets::Wrap { trim: false })
                                .render(inner, buf);
                        }
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
