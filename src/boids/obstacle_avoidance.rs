use obstacles::Obstacle;

use super::*;

pub struct ObstacleAvoidancePlugin;

impl Plugin for ObstacleAvoidancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (obstacle_detection, obstacle_avoidance)
                .chain()
                .in_set(ObstacleAvoidanceSet),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObstacleAvoidanceSet;

#[derive(Component)]
pub struct ObstacleAvoidance;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct AvoidObstacle;

fn obstacle_avoidance(
    mut q_boids: Populated<BoidsQuery, With<AvoidObstacle>>,
    vision_radius: Res<VisionRadius>,
    spatial_query: SpatialQuery,
    config: Res<SimulationConfig>,
    mut detection_angles: Local<Vec<Vec2>>,
    mut step_angle: Local<f32>,
) {
    if config.is_changed() {
        *step_angle =
            std::f32::consts::TAU / (2. * (config.obstacle_detection_density as f32 + 1.));
        let mut new_angles = Vec::new();
        for i in 1..(config.obstacle_detection_density + 1) {
            let cast_angle = i as f32 * *step_angle;
            let new_dir_right = Vec2::Y.rotate_towards(-Vec2::Y, cast_angle);
            let new_dir_left = Vec2::Y.rotate_towards(-Vec2::Y, -cast_angle);
            new_angles.push(new_dir_right);
            new_angles.push(new_dir_left);
        }
        *detection_angles = new_angles;
    }

    for mut boid in q_boids.iter_mut() {
        let rotated_directions = detection_angles
            .iter()
            .map(|v| boid.transform.rotation.mul_vec3(v.extend(0.)).xy())
            .collect::<Vec<_>>();
        let mut distance_to_obstacle: f32 = 1.;

        for dir in rotated_directions.iter() {
            let hit_test = spatial_query.cast_ray(
                boid.transform.translation.xy(),
                Dir2::new(*dir).expect("The vec shouldn't be of len 0, infinite or nan"),
                vision_radius.0,
                false,
                &SpatialQueryFilter::from_mask(GameCollisionLayer::Obstacles),
            );

            // We found a "free" direction and should steer towards it
            if hit_test.is_none() {
                let desired_steer = dir.normalize() - boid.vel.xy().normalize();
                boid.dir.0 += desired_steer.normalize() / distance_to_obstacle
                    * config.obstacle_avoidance_strength;

                // Stopping the checks on this one
                break;
            }

            // Otherwise, we keep testing
            distance_to_obstacle = distance_to_obstacle.min(hit_test.expect("some test").distance);
        }

        // If we tested all directions and found nothing, the best course of action
        // in my opinion is to just decelerate, since all we know at that point is that
        // there's some obstacle ahead of us
        boid.dir.0 += -boid.vel.xy().normalize() * config.obstacle_avoidance_strength;
    }
}

fn obstacle_detection(
    q_boids: Populated<BoidsQueryReadOnly, With<ObstacleAvoidance>>,
    mut commands: Commands,
    q_vision_cones: Query<BoidVisionQuery>,
    q_obstacles: Query<Entity, With<Obstacle>>,
    spatial_query: SpatialQuery,
    vision_radius: Res<VisionRadius>,
) {
    for vision_cone in q_vision_cones.iter() {
        if vision_cone.colliding.is_empty() {
            continue;
        }

        let seen_obstacles = vision_cone
            .colliding
            .iter()
            .filter(|e| q_obstacles.contains(**e))
            .collect::<Vec<_>>();

        if seen_obstacles.is_empty() {
            continue;
        }

        let boid = q_boids
            .get(vision_cone.parent.get())
            .expect("Should get boid");

        let hit_test = spatial_query.cast_ray(
            boid.transform.translation.xy(),
            Dir2::new(boid.vel.xy()).expect("The vec shouldn't be of len 0, infinite or nan"),
            vision_radius.0 / 2.,
            false,
            &SpatialQueryFilter::from_mask(GameCollisionLayer::Obstacles),
        );

        if hit_test.is_none() {
            commands.entity(boid.entity).remove::<AvoidObstacle>();
            continue;
        }

        commands.entity(boid.entity).insert(AvoidObstacle);
    }
}
