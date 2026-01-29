use std::{sync::mpsc, thread};

use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame, layout::Constraint, style::Stylize, symbols::{border}, text::Line, widgets::{Block, Padding, Row, Table, Widget}};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = ratatui::init();

    let mut app = App {
        exit: false,
        processes: Vec::new()
    };

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
}

enum Event {
    Input(crossterm::event::KeyEvent),
    Processes(Vec<ProcessInfo>)
}

struct ProcessInfo {
    pid: i32,
    name: String,
    cpu_usage: f32
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
            " q ".blue().bold()
        ]);
    
        let block = Block::bordered()
            .padding(Padding::new(2, 2, 1, 1))
            .title(title)
            .title_bottom(instruct)
            .border_set(border::THICK);
    
        let rows = self.processes.iter().map(|p| {
            Row::new(vec![
                p.pid.to_string(),
                p.name.clone(),
                format!("{:.2}", p.cpu_usage)
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(8),      // pid
                Constraint::Min(65), // bname
                Constraint::Length(8),      // cpu_usage
            ],
        )
        .header(
            Row::new(vec!["PID", "NAME", "CPU%"])
                .bold()
                .bottom_margin(1),
        )
        .block(block)
        .column_spacing(1);

        table.render(area, buf);
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
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_cpu_usage();

        let processes = sys
            .processes()
            .iter()
            .map(|(pid, proc)| ProcessInfo {
                pid: pid.as_u32() as i32,
                name: proc.name().to_str().unwrap().to_string(),
                cpu_usage: proc.cpu_usage()
            })
            .collect::<Vec<_>>();

        tx.send(Event::Processes(processes)).ok();

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}
