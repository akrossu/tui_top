use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Cell, Padding, Row, Scrollbar, Table},
};

use crate::{app::app::App, system::processes::COLUMNS};

/// Renders the process table (with header, sort indicators, key-binding hints,
/// row highlighting, and a vertical scrollbar) into the given area.
pub fn render_process_table(app: &mut App, frame: &mut Frame, area: Rect) {
    let title = Line::from(" TuiTop Process Manager ").bold();
    let sort_direction = if app.sort_desc { " Sort Asc " } else { " Sort Desc " };

    let instruct = Line::from(vec![
        " Quit ".into(),
        "<q> ".blue().bold(),
        "-".into(),
        " Select ".into(),
        "<↑, ↓> ".blue().bold(),
        "-".into(),
        " Sort by ".into(),
        "<←, →> ".blue().bold(),
        "-".into(),
        sort_direction.into(),
        "<s> ".blue().bold(),
    ]);

    let block = Block::bordered()
        .padding(Padding::new(2, 2, 1, 1))
        .title(title)
        .title_bottom(instruct)
        .border_set(border::ROUNDED);

    let inner = block.inner(area);

    let layout = Layout::horizontal([Constraint::Min(0), Constraint::Length(1)]).split(inner);

    // --- Table rows ---
    let rows = app.processes.iter().map(|p| {
        Row::new(vec![
            p.pid.to_string(),
            p.name.clone(),
            format!("{:.02}", p.cpu_usage),
            format!("{:.02} MB", p.ram_usage as f64 / 1024.0 / 1024.0),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // pid
            Constraint::Min(65),    // name
            Constraint::Length(8),  // cpu_usage
            Constraint::Length(12), // ram_usage
        ],
    )
    .header(
        Row::new(COLUMNS.iter().enumerate().map(|(i, col)| {
            let mut title = col.title.to_string();
            if i == app.sort_column {
                title.push(if app.sort_desc { '↓' } else { '↑' });
            }
            Cell::from(title)
        }))
        .bold()
        .bg(Color::Rgb(100, 133, 88)),
    )
    .block(block)
    .column_spacing(1)
    .row_highlight_style(Style::default().bg(Color::Blue).bold());

    frame.render_stateful_widget(table, area, &mut app.table_state);

    // --- Scrollbar ---
    let total = app.processes.len();
    let selected = app.table_state.selected().unwrap_or(0);

    app.scrollbar_state = app
        .scrollbar_state
        .content_length(total)
        .position(selected);

    let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().bg(Color::Gray))
        .track_style(Style::default().fg(Color::DarkGray));

    frame.render_stateful_widget(scrollbar, layout[1], &mut app.scrollbar_state);
}