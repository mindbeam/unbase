
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn random_2d (z: f32) -> Self {
        let mut rng = thread_rng();

        Position{
            x: 2000 * rng.gen() - 1000,
            y: 2000 * rng.gen() - 1000,
            z: z,
        }
    }
    pub fn random_3d () -> Self {
        let mut rng = thread_rng();

        Position{
            x: 2000 * rng.gen() - 1000,
            y: 2000 * rng.gen() - 1000,
            z: 2000 * rng.gen() - 1000,
        }
    }
}