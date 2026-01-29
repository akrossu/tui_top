use std::{sync::mpsc, thread};

use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame, layout::{Constraint, Layout}, style::{Color, Style, Stylize}, symbols::border, text::Line, widgets::{Block, Cell, Padding, Paragraph, Row, Table, Widget}};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let mut app = App {
        exit: false,
        processes: Vec::new(),
        system_info: SystemInfo { uptime: 0 }
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
    system_info: SystemInfo
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

enum Event {
    Input(crossterm::event::KeyEvent),
    Processes(Vec<ProcessInfo>),
    SystemInfo(SystemInfo)
}

impl App {
    fn run(&mut self, terminal:&mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> Result<()> {
        while !self.exit {
            while let Ok(event) = rx.try_recv() {
                match event {
                    Event::Input(key) => {
                        if key.code == crossterm::event::KeyCode::Char('q') {
                            self.exit = true;
                        }
                    }

                    Event::Processes(procs) => {
                        self.processes = procs;
                    }

                    Event::SystemInfo(info) => {
                        self.system_info = info;
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame:&mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

// need &App for scoping. Because &App is an immutable reference.
// eg. can not change self.exit in this context
impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) where Self: Sized {
        let title = Line::from(" TuiTop Process Manager ").bold();
    
        let instruct = Line::from(vec![
            " Quit ".into(),
            "<q> ".blue().bold(),
            "-".into(),
            " Select ".into(),
            "<↑, ↓> ".blue().bold(),
            "-".into(),
            " Sort by ".into(),
            "<←, →> ".blue().bold()
        ]);
    
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(0)
            ])
            .split(area);


        // -----------------------------------------------------------------------------------------------
        let info = Paragraph::new(vec![
            Line::from(format!("Uptime: {}", format_time(self.system_info.uptime)))
        ])
        .block(Block::bordered().title(" System ").border_set(border::ROUNDED).padding(Padding::new(1, 1, 1, 1)));

        info.render(chunks[0], buf);

        // -----------------------------------------------------------------------------------------------
        let block = Block::bordered()
            .padding(Padding::new(2, 2, 1, 1))
            .title(title)
            .title_bottom(instruct)
            .border_set(border::ROUNDED);
    
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
            Row::new(vec![
                    Cell::from("PID").style(Style::default().bg(Color::White).fg(Color::Black)),
                    Cell::from("NAME"),
                    Cell::from("CPU%"),
                    Cell::from("MEM")
                ])
                .bold()
                .bg(Color::Rgb(100, 133, 88))
                // .bg(Color::Rgb(107, 88, 133))
        )
        .block(block)
        .column_spacing(1);

        table.render(chunks[1], buf);
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
    // let sys = System::new_all();

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