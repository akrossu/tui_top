use ratatui::{Frame, layout::Rect, symbols::border, text::Line, widgets::{Block, Padding, Paragraph}};

use crate::app::app::App;
use crate::utils::time_utils::format_time;

/// Renders the system-info header panel (uptime, etc.) into the given area.
pub fn render_system_info(app: &App, frame: &mut Frame, area: Rect) {
    let info = Paragraph::new(vec![
        Line::from(format!("Uptime: {}", format_time(app.system_info.uptime))),
    ])
    .block(
        Block::bordered()
            .title(" System ")
            .border_set(border::ROUNDED)
            .padding(Padding::new(1, 1, 1, 1)),
    );

    frame.render_widget(info, area);
}