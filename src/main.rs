mod app;
mod system;
mod interfaces;
mod utils;

use app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let event_rx = system::threads::spawn_threads();

    let mut app = App::default();
    let result= app.run(&mut terminal, event_rx);

    ratatui::restore();
    return result;
}