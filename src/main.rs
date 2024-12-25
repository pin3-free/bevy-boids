use boids::{BoidSpawn, BoidsPlaygroundPlugin, BoidsPlugin};

use crate::prelude::*;

mod prelude {
    pub use avian2d::prelude::*;
    pub use bevy::prelude::*;
    pub use rand::Rng;

    #[derive(PhysicsLayer, Default)]
    pub enum GameCollisionLayer {
        #[default]
        Default,
        VisionCones,
        Boids,
    }
}

mod boids;

fn debug_special_boid(
    q_special: Populated<Entity, With<boids::SpecialBoid>>,
    q_vision_cones: Populated<(&CollidingEntities, &Parent), With<boids::BoidVisionCone>>,
) {
    for (colliding_entities, parent) in q_vision_cones.iter() {
        if q_special.get(parent.get()).is_ok() {
            info!("Special boid sees {} boids", colliding_entities.len());
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
            BoidsPlugin {
                evasion: false,
                max_force: 20.,
                max_speed: 150.,
                vision_radius: 100.,
                separation_strength: 50.,
            },
            BoidsPlaygroundPlugin {
                x_count: 5,
                y_count: 5,
                x_gap: 150.,
                y_gap: 150.,
            },
        ))
        .run();
}
