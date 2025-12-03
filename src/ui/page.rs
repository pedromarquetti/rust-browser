use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

use crate::client::parser::ParsedPage;
use crate::state::State;

#[derive(Debug, Default)]
pub struct Page {
    pub is_loading: bool,
    pub page: ParsedPage,
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

            let items: Vec<ListItem> = state
                .term_state
                .tab_state
                .curr_tab
                .clone()
                // TODO: make this better? this looks ugly lol
                .unwrap_or_default()
                .content
                .unwrap_or_default()
                .parsed_content
                .iter()
                .enumerate()
                .map(|(_, part)| {
                    // creating List
                    part.to_list_item(available_width)
                })
                .collect();

            let list = List::new(items.clone()).block(block).highlight_symbol(">");

            let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(area);

            if let Some(tab) = &mut state.term_state.tab_state.curr_tab {
                if let Some(content) = &mut tab.content {
                    StatefulWidget::render(list, list_area, buf, &mut content.state);
                }
            }
        } else {
            Paragraph::new("Loading...")
                .centered()
                .block(Block::new().borders(Borders::all()))
                .render(area, buf);
        }
    }
}
