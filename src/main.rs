use std::{cmp::Ordering, sync::mpsc, thread};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Cell, Padding, Paragraph, Row, Scrollbar, ScrollbarState, Table, TableState}};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let mut app = App {
        exit: false,
        processes: Vec::new(),
        system_info: SystemInfo { uptime: 0 },
        table_state: TableState::default(),
        scrollbar_state: ScrollbarState::default(),
        sort_column: 0,
        sort_desc: true,
    };
    get_sys_info(&mut app.system_info);

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || handle_input_events(tx_to_input_events));

    let tx_to_background_events = event_tx.clone();
    thread::spawn(move || run_background_thread(tx_to_background_events));

    let result= app.run(&mut terminal, event_rx);

    ratatui::restore();
    return result;
}

pub struct App {
    exit: bool,
    processes: Vec<ProcessInfo>,
    system_info: SystemInfo,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    sort_column: usize,
    sort_desc: bool,
}

struct SystemInfo {
    uptime: u64
}

struct ProcessInfo {
    pid: i32,
    name: String,
    cpu_usage: f32,
    ram_usage: u64
}

struct Column {
    id: &'static str,
    title: &'static str,
    cmp: fn(&ProcessInfo, &ProcessInfo) -> Ordering,
}

enum Event {
    Input(crossterm::event::KeyEvent),
    Processes(Vec<ProcessInfo>),
    SystemInfo(SystemInfo)
}

static COLUMNS: &[Column] = &[
    Column {
        id: "pid",
        title: "PID",
        cmp: |a, b| a.pid.cmp(&b.pid),
    },
    Column {
        id: "name",
        title: "NAME",
        cmp: |a, b| a.name.cmp(&b.name),
    },
    Column {
        id: "cpu",
        title: "CPU%",
        cmp: |a, b| {
            a.cpu_usage
                .partial_cmp(&b.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        },
    },
    Column {
        id: "mem",
        title: "MEM",
        cmp: |a, b| a.ram_usage.cmp(&b.ram_usage),
    },
];


impl App {
    fn run(&mut self, terminal:&mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> Result<()> {
        while !self.exit {
            self.handle_events(&rx);
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn handle_events(&mut self, rx: &mpsc::Receiver<Event>) {
        while let Ok(event) = rx.try_recv() {
            match event {
                Event::Input(key) => self.handle_key(key),
                Event::Processes(procs) => {
                    self.processes = procs;
                    self.sort_processes();
                }
                Event::SystemInfo(info) => self.system_info = info,
            }
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Down => {
                let i = match self.table_state.selected() {
                    Some(i) => {
                        if i + 1 >= self.processes.len() {
                            i
                        }
                        else {
                            i + 1
                        }
                    }
                    None => 0
                };
                self.table_state.select(Some(i));
            },
            KeyCode::Up => {
                let i = match self.table_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => 0
                };
                self.table_state.select(Some(i));
            },
            KeyCode::Left => {
                self.sort_column = (self.sort_column + COLUMNS.len() - 1) % COLUMNS.len();
            },
            KeyCode::Right => {
                self.sort_column = (self.sort_column + 1) % COLUMNS.len();
            }
            KeyCode::Char('s') => {
                self.sort_desc = !self.sort_desc;
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame:&mut Frame) {
        // frame.render_widget(self, frame.area());
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0)
            ])
            .split(frame.area());

        self.render_system_info(frame, chunks[0]);
        self.render_process_table(frame, chunks[1]);
    }

    fn render_system_info(&mut self, frame: &mut Frame, area: Rect) {
        let info = Paragraph::new(vec![
            Line::from(format!("Uptime: {}", format_time(self.system_info.uptime)))
        ])
        .block(Block::bordered()
            .title(" System ")
            .border_set(border::ROUNDED)
            .padding(Padding::new(1, 1, 1, 1))
        );

        frame.render_widget(info, area);
    }

    fn render_process_table(&mut self, frame: &mut Frame, area: Rect) {        
        let title = Line::from(" TuiTop Process Manager ").bold();
        let sort_direction = if self.sort_desc { " Sort Asc " } else { " Sort Desc " };
    
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
            "<s> ".blue().bold()
        ]);

        let block = Block::bordered()
            .padding(Padding::new(2, 2, 1, 1))
            .title(title)
            .title_bottom(instruct)
            .border_set(border::ROUNDED);
    
        let inner = block.inner(area);

        let layout = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(1)
        ])
        .split(inner);

        // Table
        let rows = self.processes.iter().map(|p| {
            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format!("{:.02}", p.cpu_usage),
                format!("{:.02} MB", p.ram_usage as f64 / 1024.0 / 1024.0)
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),  // pid
                Constraint::Min(65),    // bname
                Constraint::Length(8),  // cpu_usage
                Constraint::Length(12)  // ram_usage
            ],
        )
        .header(
            Row::new(COLUMNS.iter().enumerate().map(|(i, col)| {
                    let mut title = col.title.to_string();
                    if i == self.sort_column {
                        title.push(if self.sort_desc { '↓' } else { '↑' });
                    }
                    Cell::from(title)
                }))
            .bold()
            .bg(Color::Rgb(100, 133, 88))
        )
        .block(block)
        .column_spacing(1)
        .row_highlight_style(Style::default().bg(Color::Blue).bold());
        
        frame.render_stateful_widget(
            table,
            area,
            &mut self.table_state,
        );

        // Scrollbar
        let total = self.processes.len();
        let selected = self.table_state.selected().unwrap_or(0);

        self.scrollbar_state = self
            .scrollbar_state
            .content_length(total)
            .position(selected);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().bg(Color::Gray))
            .track_style(Style::default().fg(Color::DarkGray));

        frame.render_stateful_widget(
            scrollbar,
            layout[1],
            &mut self.scrollbar_state,
        );
    }

    fn sort_processes(&mut self) {
        let column = &COLUMNS[self.sort_column];
        let desc = self.sort_desc;

        self.processes.sort_by(|a, b| {
            let ord = (column.cmp)(a, b);
            if desc { ord.reverse() } else { ord }
        });
    }

}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(Event::Input(key_event)).unwrap();
            }
            _ => {}
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        let info = SystemInfo { uptime: System::uptime() };

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_cpu_usage();

        let processes = sys
            .processes()
            .iter()
            .map(|(pid, proc)| ProcessInfo {
                pid: pid.as_u32() as i32,
                name: proc.name().to_str().unwrap().to_string(),
                cpu_usage: proc.cpu_usage(),
                ram_usage: proc.memory()
            })
            .collect::<Vec<_>>();

        tx.send(Event::Processes(processes)).ok();
        tx.send(Event::SystemInfo(info)).ok();

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}

fn get_sys_info(system_info:&mut SystemInfo) {
    system_info.uptime = System::uptime();
}

fn seconds_to_days(mut seconds: u64) -> (u64, u64, u64, u64) {
    let days = seconds / 86400;
    seconds %= 86400;

    let hours = seconds / 3600;
    seconds %= 3600;

    let minutes = seconds / 60;
    let seconds = seconds % 60;

    (days, hours, minutes, seconds)
}

fn format_time(seconds: u64) -> String {
    let (d, h, m, s) = seconds_to_days(seconds);

    let dy = if d == 1 {
        "day"
    }
    else {
        "days"
    };

    format!("{d} {dy}, {h:02}:{m:.02}:{s:.02}")
}