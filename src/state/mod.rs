use anyhow::{Result, anyhow};
use ratatui::widgets::ListItem;
use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::error;

use crate::{
    client::{WebClientTrait, fetch_url::FetchUrl, parser::ParsedPage},
    config::Configs,
    state::{
        input::{InputState, InputType},
        tab_state::Tab,
        term::{Mode, PopupData, PopupState, TermState},
        webclient_state::{SearchProvider, WebClientState},
    },
    ui::popup_term::TermType,
};

pub mod input;
pub mod tab_state;
pub mod term;
pub mod webclient_state;

pub trait ListTrait {
    fn to_list_item(&self, width: u16) -> ListItem<'static>;
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Loaded { tab_id: i32, page: ParsedPage },
    LoadError { tab_id: i32, error: String },
}

#[derive(Debug, Clone)]
pub enum TaskType {
    /// search using defined search engine
    Search(String),
    /// go to direct URL
    Url(Url),
}

#[derive(Debug)]
/// Main App State
pub struct State {
    pub term_state: TermState,
    pub web_client_state: WebClientState,
    /// sender channel
    pub task_tx: UnboundedSender<TaskResult>,
    /// receiver channel
    pub task_rx: UnboundedReceiver<TaskResult>,
}

impl Default for State {
    fn default() -> Self {
        let (task_tx, task_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            task_rx,
            task_tx,
            term_state: TermState::default(),
            web_client_state: WebClientState::default(),
        }
    }
}

impl State {
    /// main function for updating / loading configs
    pub fn load_configs(&mut self, configs: Configs) {
        self.web_client_state.search_provider = SearchProvider {
            name: configs.webclient_config.provider,
            url: configs.webclient_config.search_url,
        }
    }

    pub fn get_tab(&mut self) -> Result<&mut Tab> {
        match self.term_state.tab_state.curr_tab_mut() {
            Some(tab) => Ok(tab),
            None => Err(anyhow!("No tab!")),
        }
    }

    pub fn handle_up(&mut self) -> Result<()> {
        if let Some(popup) = self.term_state.pop_up.as_mut() {
            match popup.popup_type.get_data() {
                PopupData::Text(_) => {
                    if self.term_state.scroll_idx != 0 {
                        self.term_state.scroll_idx -= 1
                    }
                    return Ok(());
                }
                PopupData::Links(_) => {
                    popup.list_state.select_previous();
                    return Ok(());
                }
            }
        }

        let tab = match self.get_tab() {
            Ok(tab) => tab,
            Err(_) => return Ok(()),
        };

        match tab.content_type {
            TaskType::Search(_) => self.prev_item()?,
            TaskType::Url(_) => self.scroll_up()?,
        };
        Ok(())
    }

    pub fn handle_down(&mut self) -> Result<()> {
        if let Some(popup) = self.term_state.pop_up.as_mut() {
            popup.list_state.select_next();
            return Ok(());
        }

        if self.term_state.pop_up.is_some() {
            self.term_state.scroll_idx += 1;
            return Ok(());
        }

        let tab = match self.get_tab() {
            Ok(tab) => tab,
            Err(_) => return Ok(()),
        };

        match tab.content_type {
            TaskType::Search(_) => self.next_item()?,
            TaskType::Url(_) => self.scroll_down()?,
        };
        Ok(())
    }

    pub fn prev_search(&mut self) -> Result<()> {
        if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
            if let Some(page) = tab.content.as_mut() {
                if page.curr_search_idx != 0 {
                    let curr_idx = page.curr_search_idx;
                    match page.pos.get(curr_idx as usize - 1) {
                        Some(i) => {
                            tab.scroll_idx = i.line as u16;
                            page.curr_search_idx -= 1;
                        }
                        None => {
                            self.create_popup(TermType::err(PopupData::Text(format!(
                                "No prev item!"
                            ))));
                            return Ok(());
                        }
                    }
                } else {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub fn next_search(&mut self) -> Result<()> {
        if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
            if let Some(page) = tab.content.as_mut() {
                if !page.pos.is_empty() {
                    let curr_idx = page.curr_search_idx;
                    match page.pos.get(curr_idx as usize + 1) {
                        Some(i) => {
                            tab.scroll_idx = i.line as u16;
                            page.curr_search_idx += 1;
                        }
                        None => {
                            self.create_popup(TermType::err(PopupData::Text(format!(
                                "No next item!"
                            ))));
                            return Ok(());
                        }
                    }
                } else {
                    self.create_popup(TermType::err(PopupData::Text(format!("Empty list!"))));
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Helper func. for select next list item for ParsedPage content
    fn prev_item(&mut self) -> Result<()> {
        if let Some(tab) = &mut self.term_state.tab_state.curr_tab_mut() {
            // early return if page did not finish loading
            if tab.is_loading {
                return Ok(());
            }

            if let Some(curr_tab) = &mut tab.content {
                curr_tab.state.select_previous();
            } else {
                return Err(anyhow!("no content for prev item"));
            }
        }

        Ok(())
    }

    /// Helper func. for select next list item for ParsedPage content
    fn next_item(&mut self) -> Result<()> {
        if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
            // early return if page did not finish loading
            if tab.is_loading {
                return Ok(());
            }

            if let Some(curr_tab) = &mut tab.content {
                curr_tab.state.select_next();
            } else {
                return Err(anyhow!("no content for next item"));
            }
        }

        Ok(())
    }

    fn scroll_down(&mut self) -> Result<()> {
        let term_lines = self.term_state.lines;
        if let Ok(tab) = self.get_tab().as_mut() {
            if let Some(page) = tab.content.as_mut() {
                // uses number of lines in page to determine a scroll limit
                if tab.scroll_idx <= page.linecount as u16 + term_lines + 4 {
                    tab.scroll_idx += 1;
                }
            }
        }

        Ok(())
    }

    fn scroll_up(&mut self) -> Result<()> {
        let tab = self.get_tab()?;
        if tab.scroll_idx != 0 {
            tab.scroll_idx -= 1;
        }
        Ok(())
    }

    pub fn create_popup(&mut self, popup_type: TermType) {
        self.term_state.pop_up = Some(PopupState::new(popup_type))
    }

    pub fn close_popup(&mut self) {
        self.term_state.pop_up = None;
    }

    pub fn close_app(mut self) {
        self.term_state.exit = true
    }

    /// main basic input field creator
    pub fn new_input(&mut self, input_type: InputType) {
        self.term_state.mode = Mode::Insert;
        self.term_state.input_state = Some(InputState::new(input_type));
    }

    pub fn cancel_input(&mut self) {
        self.term_state.mode = Mode::Normal;
        self.term_state.input_state = None
    }

    /// handler for processing task result from background tasks
    pub fn process_task_results(&mut self) {
        while let Ok(res) = self.task_rx.try_recv() {
            match res {
                TaskResult::Loaded { tab_id, page } => {
                    if let Err(e) = self.term_state.tab_state.update_tab_content(tab_id, page) {
                        self.create_popup(TermType::err(PopupData::Text(format!(
                            "Failed to update tab {}",
                            e
                        ))));
                    }
                }
                TaskResult::LoadError { tab_id, error } => {
                    self.create_popup(TermType::err(PopupData::Text(format!(
                        "Failed to load tab: {},\nmsg: {} ",
                        tab_id, error
                    ))));
                }
            }
        }
    }

    pub fn go_to_url(&mut self, url: Url) -> Result<()> {
        let tab_id = self.term_state.tab_state.new_tab(
            format!("loading {}", url.clone()),
            TaskType::Url(url.clone()),
        )?;
        self.spawn_page(TaskType::Url(url), tab_id)
    }

    pub fn spawn_page(&mut self, task_type: TaskType, tab_id: i32) -> Result<()> {
        let tx = self.task_tx.clone();
        let search_url = self.web_client_state.search_provider.url.clone();
        let provider = self.web_client_state.search_provider.name;
        tokio::spawn(async move {
            let mut web_state = WebClientState {
                search_provider: SearchProvider {
                    url: search_url,
                    name: provider,
                },
                ..Default::default()
            };
            let res = match task_type {
                TaskType::Search(query) => match web_state.search(query, tab_id).await {
                    Ok(page) => TaskResult::Loaded { tab_id, page: page },
                    Err(e) => {
                        error!("{:#?}", e);
                        TaskResult::LoadError {
                            tab_id,
                            error: e.to_string(),
                        }
                    }
                },
                TaskType::Url(url) => match FetchUrl::new(url.clone()).fetch_url(url, tab_id).await
                {
                    Ok(page) => TaskResult::Loaded { tab_id, page },
                    Err(e) => {
                        error!("{:#?}", e);
                        TaskResult::LoadError {
                            tab_id,
                            error: e.to_string(),
                        }
                    }
                },
            };
            match tx.send(res) {
                Ok(_) => Ok(()),
                Err(err) => Err(anyhow!("Error spawning page: {}", err)),
            }
        });
        Ok(())
    }
}
