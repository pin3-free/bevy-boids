use crate::prelude::*;

use super::{targets::SeekTarget, BoidVisionQuery, BoidsQuery, SimulationConfig};

pub struct SeekPlugin;

impl Plugin for SeekPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, seek_behaviour)
            .add_plugins(MeshPickingPlugin::default());
    }
}

#[derive(Component)]
pub struct Seek;

fn seek_behaviour(
    mut q_boids: Populated<BoidsQuery, With<Seek>>,
    q_targets: Populated<&Transform, With<SeekTarget>>,
    q_vision_cones: Populated<BoidVisionQuery>,
    config: Res<SimulationConfig>,
) {
    for vision_cone in q_vision_cones.iter() {
        if vision_cone.colliding.is_empty() {
            continue;
        }

        let seen_targets = vision_cone
            .colliding
            .iter()
            .filter_map(|ent| q_targets.get(*ent).ok())
            .collect::<Vec<_>>();

        if seen_targets.is_empty() {
            continue;
        }

        let parent_ent = vision_cone.parent.get();
        let mut boid = q_boids
            .get_mut(parent_ent)
            .expect("The boid should be present");

        let closest_target_tr = seen_targets
            .iter()
            .reduce(|acc, e| {
                let distance = boid.transform.translation.distance(e.translation);
                let cur_best_distance = boid.transform.translation.distance(acc.translation);
                if distance < cur_best_distance {
                    e
                } else {
                    acc
                }
            })
            .expect("At least one target should be found");

        let desired_vel = (closest_target_tr.translation - boid.transform.translation)
            .truncate()
            .normalize()
            * config.max_speed;

        boid.dir.0 += desired_vel - boid.vel.xy();
    }
}
