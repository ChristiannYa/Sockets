#[cfg(test)]
pub mod test;

#[macro_export]
macro_rules! logt {
    ($($arg:tt)*) => {
        println!(
            "[{}] {}",
            ::chrono::Local::now().format("%H:%M:%S%.3f"),
            format_args!($($arg)*)
        )
    };
}
