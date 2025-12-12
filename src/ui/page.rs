use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

use crate::client::parser::ParsedPage;
use crate::state::State;

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
        if !self.is_loading {
            let block = Block::default().borders(Borders::all()).title("Page");
            let inner = block.inner(area);
            let available_width = inner.width;
            let scroll_idx: i32 = state.term_state.scroll_idx;

            let (content_state, parts) = match state.term_state.tab_state.curr_tab.as_mut() {
                Some(tab) => match tab.content.as_mut() {
                    Some(content) => (&mut content.state, &content.parsed_content),
                    None => {
                        return;
                    }
                },
                None => {
                    return;
                }
            };

            let items: Vec<ListItem> = parts
                .iter()
                .enumerate()
                .map(|(_, part)| {
                    // creating List
                    part.to_list_item(available_width)
                })
                .collect();

            let list = List::new(items.clone()).block(block).highlight_symbol(">");

            let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(area);

            // if len == 1, it's a direct URL fetch
            if list.len() == 1 {
                Paragraph::new(parts[0].clone().text.unwrap_or_default())
                    .scroll((scroll_idx as u16, 0))
                    .render(list_area, buf);
            } else {
                // BUG: Opening a new tab or switching tabs should not reset item index
                StatefulWidget::render(list, list_area, buf, content_state);
            }
        } else {
            Paragraph::new("Loading...")
                .centered()
                .block(Block::new().borders(Borders::all()))
                .render(area, buf);
        }
    }
}
