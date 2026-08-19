use std::io::Write;

pub fn init_test_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .format(|buf, record| {
            let level = record.level();
            let (r, g, b) = match level {
                log::Level::Error => (255, 0, 0),  // red
                log::Level::Warn => (255, 255, 0), // yellow
                log::Level::Info => (0, 255, 0),   // green
                log::Level::Debug => hex_to_rgb("#99a0a6"),
                log::Level::Trace => (0, 255, 255), // cyan
            };

            writeln!(buf, "\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, record.args())
        })
        .try_init();
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}
