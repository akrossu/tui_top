pub fn seconds_to_days(mut seconds: u64) -> (u64, u64, u64, u64) {
    let days = seconds / 86400;
    seconds %= 86400;

    let hours = seconds / 3600;
    seconds %= 3600;

    let minutes = seconds / 60;
    let seconds = seconds % 60;

    (days, hours, minutes, seconds)
}

pub fn format_time(seconds: u64) -> String {
    let (d, h, m, s) = seconds_to_days(seconds);

    let dy = if d == 1 { "day" } else { "days" };

    format!("{d} {dy}, {h:02}:{m:02}:{s:02}")
}