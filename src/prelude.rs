pub use avian2d::prelude::*;
pub use bevy::prelude::*;
pub use rand::Rng;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default,
    VisionCones,
    Boids,
    Targets,
}
