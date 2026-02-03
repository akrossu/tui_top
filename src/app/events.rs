use std::sync::mpsc;

use crossterm::event::{KeyCode, KeyEventKind};

use crate::system::system_info::SystemInfo;
use crate::system::processes::{ProcessInfo, COLUMNS};
use crate::utils::sort;
use super::app::App;

pub enum Event {
    Input(crossterm::event::KeyEvent),
    Processes(Vec<ProcessInfo>),
    SystemInfo(SystemInfo),
}

/// Drains all pending events from the channel and updates App state accordingly.
pub fn handle_events(app: &mut App, rx: &mpsc::Receiver<Event>) {
    while let Ok(event) = rx.try_recv() {
        match event {
            Event::Input(key) => handle_key(app, key),
            Event::Processes(procs) => {
                app.processes = procs;
                sort::sort_processes(&mut app.processes, app.sort_column, app.sort_desc);
            }
            Event::SystemInfo(info) => app.system_info = info,
        }
    }
}

/// Handles a single key-press event and mutates App state.
fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Down => {
            let i = match app.table_state.selected() {
                Some(i) => {
                    if i + 1 >= app.processes.len() {
                        i
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app.table_state.select(Some(i));
        }
        KeyCode::Up => {
            let i = match app.table_state.selected() {
                Some(i) if i > 0 => i - 1,
                _ => 0,
            };
            app.table_state.select(Some(i));
        }
        KeyCode::Left => {
            app.sort_column = (app.sort_column + COLUMNS.len() - 1) % COLUMNS.len();
        }
        KeyCode::Right => {
            app.sort_column = (app.sort_column + 1) % COLUMNS.len();
        }
        KeyCode::Char('s') => {
            app.sort_desc = !app.sort_desc;
        }
        _ => {}
    }
}