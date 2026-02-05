use crate::system::processes::{ProcessInfo, COLUMNS};

/// Sorts the process list in place based on the given column index and direction.
pub fn sort_processes(processes: &mut Vec<ProcessInfo>, sort_column: usize, sort_desc: bool) {
    let column = &COLUMNS[sort_column];

    processes.sort_by(|a, b| {
        let ord = (column.cmp)(a, b);
        if sort_desc { ord.reverse() } else { ord }
    });
}