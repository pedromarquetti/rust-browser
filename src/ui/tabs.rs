use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{StatefulWidget, Tabs, Widget},
};

use crate::state::State;

#[derive(Debug)]
pub struct TabWidget {}

impl TabWidget {
    pub fn new() -> Self {
        Self {}
    }

    pub fn create(&self, area: Rect, buf: &mut Buffer, state: &mut State) -> Result<()> {
        self.render(area, buf, state);
        Ok(())
    }
}

impl StatefulWidget for &TabWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let titles = state
            .term_state
            .tab_state
            .tab_list
            .iter()
            .map(|i| return i.title.clone());

        Tabs::new(titles.clone())
            // TODO: add tab scrolling
            .select(state.term_state.tab_state.idx as usize)
            .titles(titles)
            .render(area, buf);
    }
}
