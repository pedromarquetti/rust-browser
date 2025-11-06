use anyhow::Result;
use ratatui::restore;

use crate::{state::State, ui::Term};

pub mod client;
pub mod state;
pub mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let state = &mut State {
        ..Default::default()
    };

    let app = Term::new().run(&mut terminal, state);
    app?;
    restore();
    Ok(())
}
