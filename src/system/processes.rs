use std::cmp::Ordering;

use sysinfo::System;

pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu_usage: f32,
    pub ram_usage: u64,
}

pub struct Column {
    pub id: &'static str,
    pub title: &'static str,
    pub cmp: fn(&ProcessInfo, &ProcessInfo) -> Ordering,
}



pub static COLUMNS: &[Column] = &[
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
                .unwrap_or(Ordering::Equal)
        },
    },
    Column {
        id: "mem",
        title: "MEM",
        cmp: |a, b| a.ram_usage.cmp(&b.ram_usage),
    },
];

/// Collects all running processes from a refreshed sysinfo::System instance.
pub fn collect_processes(sys: &System) -> Vec<ProcessInfo> {
    sys.processes()
        .iter()
        .map(|(pid, proc)| ProcessInfo {
            pid: pid.as_u32() as i32,
            name: proc.name().to_str().unwrap().to_string(),
            cpu_usage: proc.cpu_usage(),
            ram_usage: proc.memory(),
        })
        .collect()
}