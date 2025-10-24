use anyhow::{Context, Result, anyhow};
use ratatui::restore;
use scraper::{Html, Selector};

use crate::{client::{fetcher::get_req, WebClient}, state::State, ui::Term};

pub mod client;
pub mod state;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    // let mut terminal = ratatui::init();
    let state = &mut State {
        ..Default::default()
    };
    state.web_client_state.search("cockatiel".to_string()).await?;
    
    println!("{:#?}",state.web_client_state.curr_page);

    // let app = Term::new().run(&mut terminal, state);
    // restore();
    // app
    Ok(())
}
