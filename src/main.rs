use anyhow::Result;
use ratatui::restore;

use crate::{
    config::Configs,
    state::State,
    ui::Term,
};

pub mod client;
pub mod config;
pub mod state;
pub mod helpers;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let app = run_app().await;

    // restoring terminal if the app crashes out!
    restore();

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
