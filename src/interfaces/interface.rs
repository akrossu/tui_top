use ratatui::{Frame, layout::{Constraint, Layout}};

use crate::app::app::App;
use super::{system_info::render_system_info, process_table::render_process_table};

/// Entry point for rendering: splits the frame into the system-info header
/// and the process-table body, then delegates to each sub-renderer.
pub fn draw(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(frame.area());

    render_system_info(app, frame, chunks[0]);
    render_process_table(app, frame, chunks[1]);
}