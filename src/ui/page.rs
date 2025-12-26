use crate::client::page_part::Part;
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
            let text = format!("words: {} lines: {}", tab.wordcount, tab.linecount);
            let details = Line::from(text).style(Style::default());

            let block = Block::default()
                .borders(Borders::all())
                .title(title)
                .title(details)
                .bg(Color::Reset);

            let inner = block.inner(area);
            let available_width = inner.width;
            let scroll_idx: i32 = tab.scroll_idx;

            let items: Vec<ListItem> = content
                .parsed_content
                .iter()
                .enumerate()
                .map(|(_, part)| {
                    // creating List
                    part.to_list_item(available_width)
                })
                .collect();

            block.render(area, buf);

            Clear.render(inner, buf);

            let list = List::new(items.clone()).highlight_symbol(">");

            let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(inner);

            // if len == 1, it's a direct URL fetch
            if list.len() == 1 {
                let mut s = content.parsed_content[0]
                    .clone()
                    .content
                    .unwrap_or_default();

                Part::to_wrapped_string(&mut s, available_width);

                tab.set_wordcount(s.wordcount);
                tab.set_linecount(s.linecount);

                Paragraph::new(s.text)
                    .scroll((scroll_idx as u16, 0))
                    .render(list_area, buf);
            } else {
                StatefulWidget::render(list, list_area, buf, &mut content.state);
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
