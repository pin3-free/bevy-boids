use alignment::{Alignment, AlignmentPlugin, AlignmentSet};
use bevy::{color::palettes::css::WHITE, ecs::query::QueryData};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};
use cohesion::{Cohesion, CohesionPlugin, CohesionSet};
use compude_shader::ComputeShaderPlugin;
use configuration::{ConfigurationPlugin, ConfigurationSet, MaxForce, MaxSpeed, VisionRadius};
use obstacle_avoidance::{ObstacleAvoidance, ObstacleAvoidancePlugin, ObstacleAvoidanceSet};
use obstacles::ObstaclesPlugin;
use seek::{Seek, SeekPlugin, SeekSet};
use separation::{Separation, SeparationPlugin, SeparationSet};
use targets::TargetPlugin;

pub use configuration::SimulationConfig;

use crate::prelude::*;

pub mod seek;

pub mod separation;

pub mod alignment;

pub mod cohesion;

pub mod targets;

pub mod configuration;

pub mod obstacle_avoidance;

pub mod obstacles;

pub mod compude_shader;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct BoidsQuery {
    boid: &'static Boid,
    pub vel: &'static mut LinearVelocity,
    pub transform: &'static Transform,
    pub dir: &'static mut SteeringDirection,
    pub entity: Entity,
}

#[derive(QueryData)]
pub struct BoidVisionQuery {
    vision_cone: &'static BoidVisionCone,
    pub colliding: &'static CollidingEntities,
    pub parent: &'static Parent,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BoidsPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceSet;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        let config = SimulationConfig::default();
        app.insert_resource(config)
            // Additional simulation plugins
            .add_plugins((
                TargetPlugin,
                ConfigurationPlugin,
                ObstaclesPlugin,
                ComputeShaderPlugin,
            ))
            // Behaviour plugins
            .add_plugins((
                SeekPlugin,
                SeparationPlugin,
                AlignmentPlugin,
                CohesionPlugin,
                ObstacleAvoidancePlugin,
            ))
            .add_systems(
                FixedUpdate,
                (
                    rotate_boids,
                    steer_boids,
                    // screenwrap_boids,
                    // reset_steer
                )
                    .chain()
                    .in_set(ServiceSet),
            )
            .add_systems(Update, (boids_gizmos,))
            // Config -> Service & Seek -> Obstacle Avoidance -> Separation -> Cohesion -> Alignment
            .configure_sets(FixedUpdate, ConfigurationSet.before(ServiceSet))
            .configure_sets(FixedUpdate, SeekSet.before(SeparationSet))
            .configure_sets(FixedUpdate, ObstacleAvoidanceSet.before(SeparationSet))
            .configure_sets(FixedUpdate, SeparationSet.after(ServiceSet))
            .configure_sets(FixedUpdate, CohesionSet.after(SeparationSet))
            .configure_sets(FixedUpdate, AlignmentSet.after(CohesionSet))
            .add_event::<SpawnBoid>()
            .add_observer(spawn_boid);
    }
}

#[derive(Component, Debug)]
#[require(Collider)]
pub struct BoidCollider;

#[derive(Component, Debug)]
#[require(Collider, Sensor, CollidingEntities)]
pub struct BoidVisionCone;

#[derive(Component, Debug)]
pub struct SteeringDirection(Vec2);

impl Default for SteeringDirection {
    fn default() -> Self {
        Self(Vec2::X)
    }
}

#[derive(Component)]
#[require(
    Transform,
    ViewVisibility,
    Visibility,
    Mesh2d,
    MeshMaterial2d<ColorMaterial>,
    RigidBody,
    SteeringDirection
)]
pub struct Boid;

#[derive(Event, Default)]
pub struct SpawnBoid {
    pub loc: Vec2,
    pub angle: f32,
    pub special: bool,
}

#[derive(Component)]
pub struct SpecialBoid;

pub fn spawn_boid(
    trigger: Trigger<SpawnBoid>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<SimulationConfig>,
) {
    let scale = 10.;
    let shape = Triangle2d::new(
        (0., scale).into(),
        (-scale / 2., -scale).into(),
        (scale / 2., -scale).into(),
    );
    let color = if trigger.special {
        Color::srgb(1., 0., 0.)
    } else {
        Color::srgb_u8(2, 128, 144)
    };
    let mesh = meshes.add(shape);
    let material = materials.add(color);
    let direction = Quat::from_rotation_z(trigger.angle)
        .mul_vec3(Vec3::X)
        .truncate();

    let boid = commands
        .spawn((
            Boid,
            Seek,
            Separation,
            Alignment,
            Cohesion,
            ObstacleAvoidance,
            SteeringDirection(direction),
            Transform::from_translation(trigger.loc.extend(0.)),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Collider::triangle(
                (0., scale).into(),
                (-scale / 2., -scale).into(),
                (scale / 2., -scale).into(),
            ),
            RigidBody::Kinematic,
            CollisionLayers::new(
                GameCollisionLayer::Boids,
                [GameCollisionLayer::VisionCones, GameCollisionLayer::Targets],
            ),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Vision Cone"),
                BoidVisionCone,
                Collider::circle(config.vision_radius),
                CollidingEntities::default(),
                CollisionLayers::new(
                    GameCollisionLayer::VisionCones,
                    [
                        GameCollisionLayer::Boids,
                        GameCollisionLayer::Targets,
                        GameCollisionLayer::Obstacles,
                    ],
                ),
                Sensor,
            ));
        })
        .id();

    if trigger.special {
        commands
            .entity(boid)
            .insert((SpecialBoid, Name::new("Special Boid")));
    }
}

pub fn boids_gizmos(
    q_special: Single<BoidsQuery, With<SpecialBoid>>,
    q_boids: Query<BoidsQuery, Without<SpecialBoid>>,
    q_vision_cones: Query<BoidVisionQuery>,
    vision_radius: Res<VisionRadius>,
    max_speed: Res<MaxSpeed>,
    mut gizmos: Gizmos,
) {
    let pos = q_special.transform.translation.truncate();
    gizmos.circle_2d(pos, vision_radius.0, WHITE);
    gizmos.arrow_2d(
        pos,
        pos + q_special.dir.0.clamp_length_max(30.),
        Color::srgba(1., 1., 0., q_special.dir.0.length()),
    );
    gizmos.arrow_2d(
        pos,
        pos + q_special.vel.xy().clamp_length(25., 25.),
        Color::srgba(0., 1., 0., q_special.vel.xy().length() / max_speed.0),
    );

    for vision_cone in q_vision_cones.iter() {
        if vision_cone.parent.get() != q_special.entity {
            continue;
        }

        for colliding_ent in vision_cone.colliding.iter() {
            if let Ok(colliding_boid) = q_boids.get(*colliding_ent) {
                let distance = (colliding_boid.transform.translation
                    - q_special.transform.translation)
                    .length();
                let lines_color =
                    Color::srgba(1., 0., 0., (vision_radius.0 - distance) / vision_radius.0);
                gizmos.line_2d(
                    pos,
                    colliding_boid.transform.translation.truncate(),
                    lines_color,
                );
            }
        }
    }
}

pub fn steer_boids(
    mut q_boids: Populated<(
        &SteeringDirection,
        &mut LinearVelocity,
        Option<&SpecialBoid>,
    )>,
    max_force: Res<MaxForce>,
    max_speed: Res<MaxSpeed>,
) {
    for (steer_direction, mut linear_velocity, special) in q_boids.iter_mut() {
        let steer_force = steer_direction.0.clamp_length_max(max_force.0);
        let acceleration = steer_force; // Consider mass = 1
        if special.is_some() {
            // info!("Max force: {:?}, max speed: {:?}", max_force.0, max_speed.0);
        }
        let new_velocity = (linear_velocity.xy() + acceleration).clamp_length_max(max_speed.0);
        *linear_velocity = LinearVelocity::from(new_velocity);
    }
}

pub fn rotate_boids(mut q_boids: Populated<(&LinearVelocity, &mut Transform), With<Boid>>) {
    for (an_vel, mut transform) in q_boids.iter_mut() {
        let new_forward = an_vel.xy().normalize();
        let new_rot = Quat::from_rotation_z(new_forward.to_angle() - std::f32::consts::FRAC_PI_2);
        transform.rotation = new_rot
    }
}

#[deprecated]
#[allow(unused)]
pub fn screenwrap_boids(
    mut q_boids: Populated<&mut Transform, With<Boid>>,
    q_window: Populated<&Window>,
) {
    let window = q_window.single();
    let window_size = window.size();
    let world_halfwidth = window_size.x / 2.;
    let world_halfheight = window_size.y / 2.;
    let make_wrap = move |val: f32| {
        let res = move |num| {
            if num > val {
                -val
            } else if num < -val {
                val
            } else {
                num
            }
        };
        res
    };
    let wrapx = make_wrap(world_halfwidth);
    let wrapy = make_wrap(world_halfheight);

    for mut transform in q_boids.iter_mut() {
        transform.translation.x = wrapx(transform.translation.x);
        transform.translation.y = wrapy(transform.translation.y);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn create_test_app() -> App {
//         let mut app = App::new();
//         app.add_plugins(MinimalPlugins);
//         app
//     }

//     #[test]
//     fn boid_spawning() {
//         let mut app = create_test_app();
//         app.add_plugins((BoidsPlugin::default(), PhysicsPlugins::default()));
//         app.world_mut().trigger(SpawnBoid {
//             loc: Vec2::new(10., 20.),
//             angle: std::f32::consts::FRAC_PI_3,
//             special: false,
//         });
//         app.update();
//         let mut transform = app
//             .world_mut()
//             .query_filtered::<&Transform, With<Boid>>()
//             .get_single(app.world());
//         assert_eq!(transform.unwrap().translation, Vec3::new(10., 20., 0.));
//     }
// }
