mod app;

use app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let mut app = App::default();
    let result = app.run_app(&mut terminal);

    ratatui::restore();
    return result;
}