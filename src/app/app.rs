use std::sync::mpsc;

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;
use ratatui::widgets::{ScrollbarState, TableState};

use crate::app::{Event, events};
use crate::interfaces;
use crate::system::{ProcessInfo, SystemInfo};

pub struct App {
    pub exit: bool,
    pub processes: Vec<ProcessInfo>,
    pub system_info: SystemInfo,
    pub table_state: TableState,
    pub scrollbar_state: ScrollbarState,
    pub sort_column: usize,
    pub sort_desc: bool,
}

impl Default for App {
    fn default() -> Self {
        App { 
            exit: false,
            processes: Vec::new(),
            system_info: SystemInfo::default(),
            table_state: TableState::default(),
            scrollbar_state: ScrollbarState::default(),
            sort_column: 0,
            sort_desc: true,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> Result<()> {
        while !self.exit {
            events::handle_events(self, &rx);
            terminal.draw(|frame| interfaces::interface::draw(self, frame))?;
        }

        Ok(())
    }
}
