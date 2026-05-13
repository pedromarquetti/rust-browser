use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use ratatui::widgets::ListItem;
use reqwest::{Client, Url};
use tokio::sync::mpsc::{Receiver, Sender};
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

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

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
    pub task_tx: Sender<TaskResult>,
    /// receiver channel
    pub task_rx: Receiver<TaskResult>,
}

impl State {
    pub fn new() -> Result<Self> {
        let (task_tx, task_rx) = tokio::sync::mpsc::channel::<TaskResult>(16);
        Ok(Self {
            task_rx,
            task_tx,
            term_state: TermState::default(),
            web_client_state: WebClientState::default(),
        })
    }

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
        let width = self.term_state.cols.saturating_sub(2);
        if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
            if let Some(page) = tab.content.as_mut() {
                if page.curr_search_idx.get() != 0 {
                    let curr_idx = page.curr_search_idx.get();
                    let res = page.pos.borrow().get(curr_idx as usize - 1).cloned();
                    match res {
                        Some(i) => {
                            tab.scroll_idx = page.visual_line_for_byte(width, i.str_byte) as u16;
                            page.curr_search_idx.set(curr_idx - 1);
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
        let width = self.term_state.cols.saturating_sub(2);
        if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
            if let Some(page) = tab.content.as_mut() {
                if !page.pos.borrow().is_empty() {
                    let curr_idx = page.curr_search_idx.get();
                    let res = page.pos.borrow().get(curr_idx as usize + 1).cloned();
                    match res {
                        Some(i) => {
                            tab.scroll_idx = page.visual_line_for_byte(width, i.str_byte) as u16;
                            page.curr_search_idx.set(curr_idx + 1);
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
                curr_tab.state.borrow_mut().select_previous();
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
                curr_tab.state.borrow_mut().select_next();
            } else {
                return Err(anyhow!("no content for next item"));
            }
        }

        Ok(())
    }

    fn scroll_down(&mut self) -> Result<()> {
        let term_lines = self.term_state.lines;
        let width = self.term_state.cols.saturating_sub(2);
        if let Ok(tab) = self.get_tab().as_mut() {
            if let Some(page) = tab.content.as_mut() {
                page.to_wrapped_string(width);
                // uses number of lines in page to determine a scroll limit
                if tab.scroll_idx
                    <= page.linecount.get().unwrap_or_default() as u16 + term_lines + 4
                {
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
                        if let Some(tab) = self // set tab loading to false if failed
                            .term_state
                            .tab_state
                            .tab_list
                            .iter_mut()
                            .find(|t| t.id == tab_id)
                        {
                            tab.is_loading = false;
                        };

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
        let tab_id = self
            .term_state
            .tab_state
            .new_tab(format!("loading {}", url), TaskType::Url(url.clone()))?;
        self.spawn_page(TaskType::Url(url), tab_id)
    }

    fn ensure_web_client(&mut self) -> Result<Arc<Client>> {
        if self.web_client_state.web_client.is_none() {
            let client = Arc::new(
                Client::builder()
                    .timeout(Duration::from_secs(30))
                    .user_agent(APP_USER_AGENT)
                    .build()?,
            );
            self.web_client_state.web_client = Some(client);
        }

        match self.web_client_state.web_client.as_ref() {
            Some(client) => Ok(Arc::clone(client)),
            None => Err(anyhow!("Web client is not initialized")),
        }
    }

    pub fn ensure_current_tab_loaded(&mut self) -> Result<()> {
        let to_load = self.term_state.tab_state.curr_tab().and_then(|tab| {
            if tab.content.is_none() && !tab.is_loading {
                Some((tab.id, tab.content_type.clone()))
            } else {
                None
            }
        });

        if let Some((tab_id, task_type)) = to_load {
            if let Some(tab) = self.term_state.tab_state.curr_tab_mut() {
                tab.is_loading = true;
                // tab.scroll_idx = 0;
            }
            self.spawn_page(task_type, tab_id)?;
        }

        Ok(())
    }

    pub fn spawn_page(&mut self, task_type: TaskType, tab_id: i32) -> Result<()> {
        let tx = self.task_tx.clone();
        let search_url = self.web_client_state.search_provider.url.clone();
        let provider = self.web_client_state.search_provider.name;
        let web_client = self.ensure_web_client()?;

        tokio::spawn(async move {
            let mut web_state = WebClientState {
                search_provider: SearchProvider {
                    url: search_url,
                    name: provider,
                },
                web_client: Some(Arc::clone(&web_client)),
                ..Default::default()
            };
            let res = match task_type {
                TaskType::Search(query) => {
                    match web_state.search(query, tab_id, web_client.as_ref()).await {
                        Ok(page) => TaskResult::Loaded { tab_id, page: page },
                        Err(e) => {
                            error!("{:#?}", e);
                            TaskResult::LoadError {
                                tab_id,
                                error: e.to_string(),
                            }
                        }
                    }
                }
                TaskType::Url(url) => match FetchUrl::new(url.clone())
                    .fetch_url(url, tab_id, &web_client)
                    .await
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
            match tx.send(res).await {
                Ok(_) => Ok(()),
                Err(err) => Err(anyhow!("Error spawning page: {}", err)),
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Result, anyhow};

    #[tokio::test]
    async fn test_process_task_result() -> Result<()> {
        use crate::client::parser::{ParsedContent, ParsedPage};
        use crate::state::{State, TaskResult, TaskType};
        use reqwest::Url;

        let mut s = State::new()?;
        let id = s
            .term_state
            .tab_state
            .new_tab("loading", TaskType::Url(Url::parse("http://example.com")?))?;

        // simulate loading tab before result arrives
        s.term_state.tab_state.curr_tab_mut().unwrap().is_loading = true;

        let page = ParsedPage {
            tab_id: id,
            title: "Loaded title".into(),
            parsed_content: ParsedContent::Text("ok".into()),
            raw_text: "ok".into(),
            ..Default::default()
        };

        s.task_tx
            .send(TaskResult::Loaded { tab_id: id, page })
            .await
            .map_err(|e| anyhow!("{}", e))?;
        s.process_task_results();

        let tab = s.term_state.tab_state.curr_tab().unwrap();
        assert!(!tab.is_loading);
        assert_eq!(tab.title, "Loaded title");
        assert!(tab.content.is_some());
        assert!(s.term_state.pop_up.is_none());
        Ok(())
    }
}
