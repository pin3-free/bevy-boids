use super::{
    targets::SeekTarget, App, BoidVisionQuery, BoidsQuery, Commands, Component, FixedUpdate,
    MeshPickingPlugin, Plugin, Populated, Res, SimulationConfig, SystemSet, Transform,
    Vec2Swizzles, With,
};
pub struct SeekPlugin;

impl Plugin for SeekPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, seek_behaviour)
            .add_plugins(MeshPickingPlugin::default());
    }
}

#[derive(Component)]
pub struct Seek;

#[derive(Component)]
pub struct Chasing;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeekSet;

fn seek_behaviour(
    mut q_boids: Populated<BoidsQuery, With<Seek>>,
    q_targets: Populated<&Transform, With<SeekTarget>>,
    q_vision_cones: Populated<BoidVisionQuery>,
    config: Res<SimulationConfig>,
    mut commands: Commands,
) {
    for vision_cone in q_vision_cones.iter() {
        // if vision_cone.colliding.is_empty() {
        //     continue;
        // }

        // let seen_targets = vision_cone
        //     .colliding
        //     .iter()
        //     .filter_map(|ent| q_targets.get(*ent).ok())
        //     .collect::<Vec<_>>();
        let seen_targets = q_targets.iter().collect::<Vec<&Transform>>();

        let parent_ent = vision_cone.parent.get();
        let mut boid = q_boids
            .get_mut(parent_ent)
            .expect("The boid should be present");

        if seen_targets.is_empty() {
            commands.entity(parent_ent).remove::<Chasing>();
            continue;
        } else {
            commands.entity(parent_ent).insert(Chasing);
        }

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

        boid.dir.0 += (desired_vel - boid.vel.xy()).normalize() * config.seek_strength;
    }
}
