use chrono::format::{DelayedFormat, StrftimeItems};
use rand::RngExt;

pub fn now() -> DelayedFormat<StrftimeItems<'static>> {
    chrono::Local::now().format("%H:%M:%S%.3f")
}

#[macro_export]
macro_rules! logt {
    ($($arg:tt)*) => {
        println!(
            "[{}] {}",
            $crate::util::now(),
            format_args!($($arg)*)
        )
    };
}

#[derive(Clone, Copy)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self { Hsv { h, s, v } }

    pub fn rand() -> Self {
        let mut rng = rand::rng();
        Hsv {
            h: rng.random(),
            s: rng.random_range(0.7..=1.0),
            v: rng.random_range(0.7..=1.0),
        }
    }
}

/// Returns a uniformly random `(x, z)` point inside a disk of radius `rad`
/// centered at the origin of the XZ plane.
pub fn rand_disk_pt_xz(rad: f32) -> (f32, f32) {
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let rad = rad * rand::random::<f32>().sqrt();
    let x: f32 = rad * angle.cos();
    let z: f32 = rad * angle.sin();
    (x, z)
}
