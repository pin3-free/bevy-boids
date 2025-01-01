use super::*;

pub struct CohesionPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CohesionSet;

impl Plugin for CohesionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, cohesion_behaviour.in_set(CohesionSet));
    }
}

#[derive(Component)]
pub struct Cohesion;

fn cohesion_behaviour(
    mut q_boids: Populated<BoidsQuery, With<Cohesion>>,
    q_vision_cones: Query<BoidVisionQuery>,
    config: Res<SimulationConfig>,
) {
    for vision_cone in q_vision_cones.iter() {
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

        let avg_pos = seen_boids
            .iter()
            .map(|other| other.transform.translation.truncate())
            .reduce(|acc, e| acc + e)
            .expect("Should get a sum velocity")
            .div_euclid(Vec2::splat(seen_boids.len() as f32));

        let accel_dir = avg_pos - boid.transform.translation.xy();
        let mut boid = q_boids.get_mut(parent_ent).expect("Should get boid");
        boid.dir.0 += accel_dir.normalize() * config.cohesion_strength;
    }
}
