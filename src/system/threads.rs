use std::{sync::mpsc, thread};

use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

use crate::app::Event;
use super::system_info::fetch_system_info;
use super::processes::collect_processes;

pub fn spawn_threads() -> mpsc::Receiver<Event> {
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || handle_input_events(tx_to_input_events));

    let tx_to_background_events = event_tx.clone();
    thread::spawn(move || run_background_thread(tx_to_background_events));

    return event_rx;
}

/// Spawns a background thread that periodically refreshes system/process data
/// and sends it over the provided channel.
pub fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        let info = fetch_system_info();

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_cpu_usage();

        let processes = collect_processes(&sys);

        tx.send(Event::Processes(processes)).ok();
        tx.send(Event::SystemInfo(info)).ok();

        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}

/// Spawns a thread that listens for terminal key events and forwards them
/// as Event::Input over the provided channel.
pub fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                tx.send(Event::Input(key_event)).unwrap();
            }
            _ => {}
        }
    }
}