use chrono::Local;

#[cfg(test)]
pub mod test;

pub fn log(msg: &str) {
    println!("[{}] {msg}", Local::now().format("%H:%M:%S%.3f"))
}
