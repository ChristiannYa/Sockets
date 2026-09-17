use rand::RngExt;

pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

pub fn hsv() -> Hsv {
    let mut rng = rand::rng();
    Hsv {
        h: rng.random(),
        s: rng.random_range(0.7..=1.0),
        v: rng.random_range(0.7..=1.0),
    }
}
