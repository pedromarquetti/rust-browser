use anyhow::{Context, Result};

#[derive(Debug, Clone, Default)]
pub struct Tab {
    pub id: i32,
    pub title: String,
    // TODO: this is a placeholder for webpages
    pub content: String,
}

impl Tab {
    pub fn new(id: i32, title: String) -> Self {
        Self {
            id,
            title,
            ..Default::default()
        }
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id
    }
}

#[derive(Debug, Default, Clone)]
pub struct TabState {
    pub tab_list: Vec<Tab>,
    pub idx: i32,
    pub curr_tab: Tab,
    pub tab_count: i32,
}

impl TabState {
    pub fn del_tab(&mut self) {
        if !self.tab_list.is_empty() {
            self.tab_list.remove(self.idx as usize);
            if self.idx as usize >= self.tab_list.len() && !self.tab_list.is_empty() {
                self.prev_tab();
            }
        }
        self.handle_idx();
    }

    fn handle_idx(&mut self) {
        // if next idx is != next tablist
        // change next item id (+1)
        for (i, tab) in self.tab_list.iter_mut().enumerate() {
            tab.id = i as i32;
        }
    }

    pub fn new_tab<S:Into<String>>(&mut self,title:S) {
        self.tab_list.push(Tab::new(
            self.tab_list.len() as i32,
            title.into()
        ))
    }

    pub fn next_tab(&mut self) -> Result<()> {
        if self.idx <= self.tab_list.len() as i32 - 2 {
            self.idx = self.idx + 1;
            let nxt = self
                .tab_list
                .iter()
                .find(|i| i.id == self.idx)
                .context("Next tab not found!")?;
            self.curr_tab = nxt.clone();
        }

        Ok(())
    }

    pub fn prev_tab(&mut self) -> Result<()> {
        if self.idx >= 1 {
            self.idx = self.idx - 1;
            let prev = self
                .tab_list
                .iter()
                .find(|i| i.id == self.idx)
                .context("Next tab not found!")?;

            self.curr_tab = prev.clone();
        }
        Ok(())
    }

    pub fn set_title<S: Into<String>>(&mut self, title: S) {
        self.curr_tab.title = title.into()
    }
}
