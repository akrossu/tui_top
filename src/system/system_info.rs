use sysinfo::System;

pub struct SystemInfo {
    pub uptime: u64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        SystemInfo { 
            uptime: 0
        }
    }
}

pub fn fetch_system_info() -> SystemInfo {
    SystemInfo {
        uptime: System::uptime(),
    }
}