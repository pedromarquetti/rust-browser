use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::{
    client::{
        page_part::Part,
        parser::{ParsedContent, ParsedPage},
    },
    state::TaskType,
};

// TODO: make all these consts configurable
const MAX_LOADED_TABS: usize = 10;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: i32,
    pub title: String,
    pub content: Option<Arc<ParsedPage>>,
    pub is_loading: bool,
    pub scroll_idx: u16,
    /// defines if tab contains Search or Direct URL page
    pub content_type: TaskType,
}

impl Tab {
    pub fn new(id: i32, title: String, tab_type: TaskType) -> Self {
        Self {
            id,
            title,
            is_loading: true,
            content_type: tab_type.clone(),
            ..Default::default()
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            id: -1,
            title: "".to_string(),
            content: None,
            is_loading: false,
            scroll_idx: 0,
            content_type: TaskType::Search("".to_string()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TabState {
    pub tab_list: Vec<Tab>,
    /// current tab index
    pub curr_idx: Option<usize>,
    pub next_idx: i32,
}

impl TabState {
    pub fn curr_tab_mut(&mut self) -> Option<&mut Tab> {
        if let Some(idx) = self.curr_idx {
            if let Some(tab) = self.tab_list.get_mut(idx) {
                return Some(tab);
            }
        };
        None
    }

    pub fn curr_tab(&self) -> Option<&Tab> {
        if let Some(idx) = self.curr_idx {
            if let Some(tab) = self.tab_list.get(idx) {
                return Some(tab);
            }
        };
        None
    }

    /// get currently selected item under ListState
    pub fn get_selected_item(&mut self) -> Result<Part> {
        if let Some(tab) = &self.curr_tab() {
            if let Some(page) = &tab.content {
                let idx = page.state.borrow_mut().selected().unwrap_or(0);
                match &page.parsed_content {
                    ParsedContent::PartList(list) => {
                        // filter list
                        match list.get(idx) {
                            Some(i) => Ok(i.clone()),
                            None => Err(anyhow!("No item!")),
                        }
                    }
                    _ => Err(anyhow!("No page!")),
                }
            } else {
                Err(anyhow!("No page!"))
            }
        } else {
            Err(anyhow!("No tab!"))
        }
    }

    pub fn del_tab(&mut self) -> Result<()> {
        let Some(item) = self.curr_idx else {
            return Ok(());
        };

        self.tab_list.remove(item);

        self.curr_idx = if self.tab_list.is_empty() {
            None
        } else if item >= self.tab_list.len() {
            Some(self.tab_list.len() - 1)
        } else {
            Some(item)
        };

        Ok(())
    }

    pub fn new_tab<S: Into<String>>(&mut self, title: S, content_type: TaskType) -> Result<i32> {
        let id = self.next_idx;
        self.next_idx += 1;

        self.tab_list.push(Tab::new(id, title.into(), content_type));
        self.curr_idx = Some(self.tab_list.len() - 1);
        Ok(id)
    }

    pub fn next_tab(&mut self) -> Result<()> {
        if let Some(item) = self.curr_idx {
            self.curr_idx = Some((item + 1) % self.tab_list.len())
        }

        Ok(())
    }

    pub fn prev_tab(&mut self) -> Result<()> {
        if let Some(item) = self.curr_idx {
            let len = self.tab_list.len();
            self.curr_idx = Some((item + len - 1) % len);
        }

        Ok(())
    }

    /// function for handling async task updates
    pub fn update_tab_content(&mut self, tab_id: i32, page: ParsedPage) -> Result<()> {
        if let Some(tab) = self.tab_list.iter_mut().find(|i| i.id == tab_id) {
            tab.title = page.title.clone();
            tab.is_loading = false;
            tab.content = Some(Arc::new(page));

            // tab cleanup
            self.evict_loaded_tabs(tab_id);

            Ok(())
        } else {
            // Err(anyhow!("Tab with id {} not foumd", tab_id))
            Ok(())
        }
    }

    fn evict_loaded_tabs(&mut self, keep_tab_id: i32) {
        let mut loaded_count = self
            .tab_list
            .iter()
            .filter(|tab| tab.content.is_some())
            .count();
        if loaded_count <= MAX_LOADED_TABS {
            return;
        }

        let current_id = self.curr_tab().map(|tab| tab.id);
        for tab in &mut self.tab_list {
            if loaded_count <= MAX_LOADED_TABS {
                break;
            }
            if tab.content.is_none() {
                continue;
            }
            if tab.id == keep_tab_id || Some(tab.id) == current_id {
                continue;
            }

            tab.content = None;
            tab.scroll_idx = 0;
            tab.is_loading = false;
            loaded_count -= 1;
        }
    }
}
