use super::{BoidVisionQuery, BoidsQuery, SimulationConfig};
use crate::prelude::*;

pub struct SeparationPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeparationSet;

impl Plugin for SeparationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, separation_behaviour.in_set(SeparationSet));
    }
}

#[derive(Component)]
pub struct Separation;

fn separation_behaviour(
    mut q_boids: Populated<BoidsQuery, With<Separation>>,
    q_vision_cones: Query<BoidVisionQuery>,
    config: Res<SimulationConfig>,
) {
    for vision_cone in q_vision_cones.iter() {
        if vision_cone.colliding.is_empty() {
            continue;
        }

        let seen_boids = vision_cone
            .colliding
            .iter()
            .filter_map(|ent| q_boids.get(*ent).ok())
            .collect::<Vec<_>>();

        if seen_boids.is_empty() {
            continue;
        }

        let parent_ent = vision_cone.parent.get();
        let boid = q_boids.get(parent_ent).expect("The boid should be present");

        let avoidance_vec = seen_boids
            .iter()
            .map(|other| {
                let distance = boid
                    .transform
                    .translation
                    .distance(other.transform.translation);
                let avoid_dir = boid.transform.translation - other.transform.translation;
                avoid_dir.truncate().normalize() / distance
            })
            .reduce(|acc, e| acc + e)
            .expect("Should get a vec");

        let mut boid = q_boids.get_mut(parent_ent).expect("Should get boid");
        boid.dir.0 += avoidance_vec.clamp_length_min(1.) * config.separation_strength;
    }
}
