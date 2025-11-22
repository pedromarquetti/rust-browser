use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Widget, Wrap,
};

use crate::client::page_part::Part;
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
                .map(|(i, part)| {
                    //creating List
                    ListItem::from(part)
                })
                .collect();

            let block = Block::default().borders(Borders::all()).title("Page");

            let list = List::new(items.clone()).block(block).highlight_symbol(">");

            let [list_area] = Layout::vertical([Constraint::Fill(1)]).areas(area);

            if let Some(tab) = &mut state.term_state.tab_state.curr_tab {
                if let Some(content) = &mut tab.content {
                    StatefulWidget::render(list, list_area, buf, &mut content.state);
                }
            }

            // Paragraph::new(format!("{:#?}\n {:#?}\n{:#?}", items, list, self.page))
            // .wrap(Wrap { trim: true })
            // .scroll((state.term_state.scroll_idx as u16, 0))
            // .render(area, buf);
        } else {
            Paragraph::new("Loading...")
                .centered()
                .block(Block::new().borders(Borders::all()))
                .render(area, buf);
        }
    }
}
