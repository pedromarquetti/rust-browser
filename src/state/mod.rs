use anyhow::{Result, anyhow};
use reqwest::Url;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    client::parser::ParsedPage,
    config::Configs,
    state::{
        input::{InputState, InputType},
        tab_state::Tab,
        term::{Mode, TermState},
        webclient_state::{SearchProvider, WebClientState},
    },
};

pub mod cursor;
pub mod input;
pub mod tab_state;
pub mod term;
pub mod webclient_state;

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

    pub fn set_tab_title(&mut self, title: String) -> Result<()> {
        let tab = self.get_tab()?;
        tab.title = title;
        Ok(())
    }

    pub fn get_tab(&mut self) -> Result<&mut Tab> {
        match self.term_state.tab_state.curr_tab.as_mut() {
            Some(tab) => Ok(tab),
            None => Err(anyhow!("No tab!")),
        }
    }

    pub fn handle_up(&mut self) -> Result<()> {
        if self.term_state.is_err
            && self.term_state.scroll_idx != 0 {
                self.term_state.scroll_idx -= 1
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
        if self.term_state.is_err {
            self.term_state.scroll_idx += 1
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

    /// Helper func. for select next list item for ParsedPage content
    fn prev_item(&mut self) -> Result<()> {
        if let Some(tab) = &mut self.term_state.tab_state.curr_tab {
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
        // BUG: scrolling too much leaves some residual text render
        if let Some(tab) = &mut self.term_state.tab_state.curr_tab {
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
        let tab = self.get_tab()?;
        tab.scroll_idx += 1;
        Ok(())
    }

    fn scroll_up(&mut self) -> Result<()> {
        let tab = self.get_tab()?;
        if tab.scroll_idx != 0 {
            tab.scroll_idx -= 1;
        }
        Ok(())
    }

    pub fn create_err<S: Into<String>>(&mut self, msg: S) {
        self.term_state.is_err = true;
        self.term_state.err_msg = msg.into();
    }

    pub fn remove_err(&mut self) {
        self.term_state.is_err = false;
        self.term_state.err_msg = String::from("");
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

    pub fn return_input(&mut self) -> Option<String> {
        self.term_state.mode = Mode::Normal;
        self.term_state.input_state = None;
        match &self.term_state.input_state {
            Some(input) => Some(input.value.clone()),
            None => {
                self.create_err("No string found".to_string());
                None
            }
        }
    }

    /// handler for processing task result from background tasks
    pub fn process_task_results(&mut self) {
        while let Ok(res) = self.task_rx.try_recv() {
            match res {
                TaskResult::Loaded { tab_id, page } => {
                    if let Err(e) = self
                        .term_state
                        .tab_state
                        .update_tab_content(tab_id, page.clone())
                    {
                        self.create_err(format!("Failed to update tab {}", e));
                    }
                }
                TaskResult::LoadError { tab_id, error } => {
                    self.create_err(format!("Failed to load tab {} {} ", tab_id, error));
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
        let web_state = self.web_client_state.clone();
        tokio::spawn(async move {
            let mut web_state = web_state.clone();
            let res = match task_type {
                TaskType::Search(query) => match web_state.search(query, tab_id).await {
                    Ok(()) => TaskResult::Loaded {
                        tab_id,
                        page: web_state.curr_page,
                    },
                    Err(e) => TaskResult::LoadError {
                        tab_id,
                        error: e.to_string(),
                    },
                },
                TaskType::Url(url) => match web_state.fetch_url(url, tab_id).await {
                    Ok(()) => TaskResult::Loaded {
                        tab_id,
                        page: web_state.curr_page,
                    },
                    Err(e) => TaskResult::LoadError {
                        tab_id,
                        error: e.to_string(),
                    },
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
