pub struct SpawnPoint {
    pub x: f32,
    pub z: f32,
}

pub fn spawn_pt() -> SpawnPoint {
    let angle = rand::random::<f32>() * std::f32::consts::TAU;
    let rad = 10.0 * rand::random::<f32>().sqrt();
    let x: f32 = rad * angle.cos();
    let z: f32 = rad * angle.sin();
    SpawnPoint { x, z }
}
