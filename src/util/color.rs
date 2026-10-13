use rand::RngExt;

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
