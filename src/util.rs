mod seq;
#[cfg(test)]
pub mod test;

pub use seq::Seq;

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
