use anyhow::{Context, Result};
use ratatui::restore;

use crate::{state::State, ui::Term};

pub mod state;
pub mod ui;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let state = &mut State {
        ..Default::default()
    };
    let app = Term::new().run(&mut terminal, state);
    restore();
    app
}
