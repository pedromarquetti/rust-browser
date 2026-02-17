use anyhow::{Result, anyhow};

use crate::{
    client::{
        page_part::Part,
        parser::{ParsedContent, ParsedPage},
    },
    state::TaskType,
};

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: i32,
    pub title: String,
    pub content: Option<ParsedPage>,
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
            content_type: tab_type,
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
pub struct TabState  {
    pub tab_list: Vec<Tab>,
    /// current tab index
    pub idx: i32,
    pub curr_tab: Option<Tab>,
}

impl TabState {
    /// Helper func. to save current tab state to tab list
    /// (scroll idx fix)
    pub fn save_tab(&mut self) {
        if let Some(tab) = &self.curr_tab
            && let Some(stored_tab) = self.tab_list.get_mut(self.idx as usize)
        {
            *stored_tab = tab.clone();
        }
    }

    /// get currently selected item under ListState
    pub fn get_selected_item(&self) -> Result<Part> {
        if let Some(tab) = &self.curr_tab {
            if let Some(page) = &tab.content {
                let idx = page.state.selected().unwrap_or(0);
                match &page.parsed_content {
                    ParsedContent::PartList(list) => {
                        // filter list
                        Ok(list[idx].clone())
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
        if self.tab_list.is_empty() {
            return Ok(());
        }

        self.tab_list.remove(self.idx as usize);
        self.fix_idx();

        if self.idx > 0 {
            self.idx -= 1;
        }

        if self.tab_list.is_empty() {
            self.curr_tab = None;
            self.idx = 0;
        } else {
            self.sync_content()?;
        }

        Ok(())
    }

    fn fix_idx(&mut self) {
        // if next idx is != next tablist
        // change next item id (+1)
        for (i, tab) in self.tab_list.iter_mut().enumerate() {
            tab.id = i as i32;
        }
    }

    pub fn new_tab<S: Into<String>>(&mut self, title: S, content_type: TaskType) -> Result<i32> {
        if self.curr_tab.as_mut().is_some() {
            self.save_tab();
        }

        let tab = Tab::new(self.tab_list.len() as i32, title.into(), content_type);
        self.tab_list.push(tab.clone());
        self.idx = tab.id;
        self.curr_tab = Some(tab.clone());
        Ok(tab.id)
    }

    /// helper function to set curr_tab with the id
    fn sync_content(&mut self) -> Result<()> {
        if let Some(tab) = self.tab_list.get(self.idx as usize) {
            self.curr_tab = Some(tab.clone())
        }
        Ok(())
    }

    pub fn next_tab(&mut self) -> Result<()> {
        if self.tab_list.is_empty() {
            return Ok(());
        }

        self.save_tab();

        self.idx = (self.idx + 1).min(self.tab_list.len() as i32 - 1);

        self.sync_content()?;

        Ok(())
    }

    pub fn prev_tab(&mut self) -> Result<()> {
        if self.tab_list.is_empty() {
            return Ok(());
        }

        self.save_tab();

        self.idx = (self.idx - 1).max(0);

        self.sync_content()?;

        Ok(())
    }

    /// function for handling async task updates
    pub fn update_tab_content(&mut self, tab_id: i32, page: ParsedPage) -> Result<()> {
        if let Some(tab) = self.tab_list.iter_mut().find(|i| i.id == tab_id) {
            tab.content = Some(page.clone());
            tab.title = page.title.clone();
            tab.is_loading = false;

            if self.idx == tab.id
                && let Some(curr_tab) = &mut self.curr_tab
            {
                curr_tab.content = Some(page);
                curr_tab.is_loading = false;
            }

            Ok(())
        } else {
            Err(anyhow!("Tab with id {} not foumd", tab_id))
        }
    }
}
