use anyhow::Result;
use ratatui::restore;
use tracing::{info,error};

use crate::{config::Configs, helpers::init_log, state::State, ui::Term};

pub mod client;
pub mod config;
pub mod helpers;
pub mod state;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = init_log()?;
    info!("App Starting");
    let app = run_app().await;

    // restoring terminal if the app crashes out!
    restore();

    if let Err(ref e ) = app  {
        error!(error = %e, "App error!");
    }else {
        info!("App exit Ok!")
    }

    app
}

async fn run_app() -> Result<()> {
    let mut terminal = ratatui::init();

    let config: Configs = Configs::new()?;

    let state = &mut State {
        ..Default::default()
    };

    state.load_configs(config);

    Term::new().run(&mut terminal, state)
}
