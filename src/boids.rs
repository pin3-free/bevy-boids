use bevy::color::palettes::css::{GREEN, RED, WHITE, YELLOW};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};

use crate::prelude::*;

pub struct BoidsPlugin {
    pub evasion: bool,
    pub max_force: f32,
    pub max_speed: f32,
    pub vision_radius: f32,
    pub separation_strength: f32,
    pub cohesion_strength: f32,
    pub alignment_strength: f32,
}

impl Default for BoidsPlugin {
    fn default() -> Self {
        Self {
            evasion: true,
            max_force: 30.,
            max_speed: 150.,
            vision_radius: 100.,
            separation_strength: 100.,
            cohesion_strength: 70.,
            alignment_strength: 10.,
        }
    }
}

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        let BoidsPlugin {
            evasion: _,
            max_force,
            max_speed,
            vision_radius,
            separation_strength,
            cohesion_strength,
            alignment_strength,
        } = *self;
        app.insert_resource(SimulationConfig {
            max_force,
            max_speed,
            vision_radius,
            separation_strength,
            cohesion_strength,
            alignment_strength,
        })
        .register_type::<SimulationConfig>()
        .add_plugins(ResourceInspectorPlugin::<SimulationConfig>::default())
        .add_systems(
            Update,
            (
                rotate_boids,
                separation_boids,
                cohesion_boids,
                alignment_boids,
                steer_boids,
                screenwrap_boids,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                seek_cursor_boids
                    .before(steer_boids)
                    .after(separation_boids),
                boids_gizmos,
            ),
        )
        .add_event::<BoidSpawn>()
        .add_observer(spawn_boid);

        if self.evasion {
            // app.add_systems(Update, (boid_avoidance).after(move_boids_forward));
        }
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

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource)]
pub struct SimulationConfig {
    max_force: f32,
    max_speed: f32,
    vision_radius: f32,
    separation_strength: f32,
    cohesion_strength: f32,
    alignment_strength: f32,
}

#[derive(Event, Default)]
pub struct BoidSpawn {
    pub loc: Vec2,
    pub angle: f32,
    pub special: bool,
}

#[derive(Component)]
pub struct SpecialBoid;

pub fn spawn_boid(
    trigger: Trigger<BoidSpawn>,
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
            CollisionLayers::new(GameCollisionLayer::Boids, [GameCollisionLayer::VisionCones]),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Vision Cone"),
                BoidVisionCone,
                Collider::circle(config.vision_radius),
                CollidingEntities::default(),
                CollisionLayers::new(GameCollisionLayer::VisionCones, [GameCollisionLayer::Boids]),
                Sensor,
            ));
        })
        .id();
    if trigger.special {
        commands.entity(boid).insert(SpecialBoid);
    }
}

pub fn boids_gizmos(
    q_special: Single<(Entity, &LinearVelocity, &SteeringDirection, &Transform), With<SpecialBoid>>,
    q_boids: Populated<&Transform, (With<Boid>, Without<SpecialBoid>)>,
    q_vision_cones: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
    config: Res<SimulationConfig>,
    mut gizmos: Gizmos,
) {
    let (ent, linear_vel, steer, tr) = *q_special;
    let pos = tr.translation.truncate();
    gizmos.circle_2d(pos, config.vision_radius, WHITE);

    for (colliding_entities, parent) in q_vision_cones.iter() {
        if parent.get() != ent {
            continue;
        }

        for colliding_ent in colliding_entities.iter() {
            let colliding_tr = q_boids.get(*colliding_ent).unwrap();
            let distance = (colliding_tr.translation - tr.translation).length();
            let lines_color = Color::srgba(
                1.,
                0.,
                0.,
                (config.vision_radius - distance) / config.vision_radius,
            );
            gizmos.line_2d(pos, colliding_tr.translation.truncate(), lines_color);
        }
    }
}

pub fn steer_boids(
    mut q_boids: Populated<(&SteeringDirection, &mut LinearVelocity)>,
    config: Res<SimulationConfig>,
) {
    for (steer_direction, mut linear_velocity) in q_boids.iter_mut() {
        let steer_force = steer_direction.0.clamp_length_max(config.max_force);
        let acceleration = steer_force; // Consider mass = 1
        let new_velocity = (linear_velocity.xy() + acceleration).clamp_length_max(config.max_speed);
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

pub fn seek_cursor_boids(
    mut q_boids: Populated<(&LinearVelocity, &Transform, &mut SteeringDirection), With<Boid>>,
    q_camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    config: Res<SimulationConfig>,
) {
    let (camera, camera_tr) = *q_camera;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(point) = camera.viewport_to_world_2d(camera_tr, cursor_pos) else {
        return;
    };

    gizmos.circle_2d(point, 10., WHITE);

    for (linear_vel, transform, mut steer) in q_boids.iter_mut() {
        let desired_vel = point - transform.translation.truncate();
        let seek_steer = (desired_vel - linear_vel.xy()).clamp_length_max(config.max_force);
        steer.0 += seek_steer / 17.5;
    }
}

pub fn separation_boids(
    mut q_boids: Populated<(&mut SteeringDirection, &LinearVelocity, &Transform), With<Boid>>,
    q_boid_vision: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
    config: Res<SimulationConfig>,
) {
    for (colliding_entities, parent) in q_boid_vision.iter() {
        if colliding_entities.len() == 0 {
            // We should only do this whole thing if there's something to collide with
            continue;
        }
        let mut separation_force = Vec2::ZERO;
        let parent_ent = parent.get();
        let (_, _, parent_tr) = q_boids.get(parent_ent).unwrap();

        for colliding_ent in colliding_entities.iter() {
            let (_, _, other_tr) = q_boids.get(*colliding_ent).unwrap();
            let sep_force_cur = (parent_tr.translation - other_tr.translation).truncate();
            separation_force = (separation_force + sep_force_cur).normalize()
                * config.separation_strength
                / config.vision_radius;
        }

        let (mut parent_steer, _, _) = q_boids.get_mut(parent_ent).unwrap();
        parent_steer.0 += separation_force;
    }
}

pub fn cohesion_boids(
    mut q_boids: Populated<(&mut SteeringDirection, &LinearVelocity, &Transform), With<Boid>>,
    q_boid_vision: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
    config: Res<SimulationConfig>,
) {
    for (colliding_entities, parent) in q_boid_vision.iter() {
        if colliding_entities.len() == 0 {
            // We should only do this whole thing if there's something to collide with
            continue;
        }

        let mut avg_pos = Vec2::ZERO;
        let parent_ent = parent.get();
        let (_, _, parent_tr) = q_boids.get(parent_ent).unwrap();

        for colliding_ent in colliding_entities.iter() {
            let (_, _, other_tr) = q_boids.get(*colliding_ent).unwrap();
            avg_pos += other_tr.translation.truncate();
        }

        avg_pos /= colliding_entities.len() as f32;

        let to_avg_steer = avg_pos - parent_tr.translation.truncate();
        let (mut parent_steer, _, _) = q_boids.get_mut(parent_ent).unwrap();
        parent_steer.0 += to_avg_steer.normalize() * config.cohesion_strength;
    }
}

pub fn alignment_boids(
    mut q_boids: Populated<(&mut SteeringDirection, &LinearVelocity, &Transform), With<Boid>>,
    q_boid_vision: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
    config: Res<SimulationConfig>,
) {
    for (colliding_entities, parent) in q_boid_vision.iter() {
        if colliding_entities.len() == 0 {
            // We should only do this whole thing if there's something to collide with
            continue;
        }

        let mut avg_vel = Vec2::ZERO;
        let parent_ent = parent.get();
        let (_, parent_vel, _) = q_boids.get(parent_ent).unwrap();

        for colliding_ent in colliding_entities.iter() {
            let (_, other_vel, _) = q_boids.get(*colliding_ent).unwrap();
            avg_vel += other_vel.xy();
        }

        avg_vel /= colliding_entities.len() as f32;

        let to_avg_vel_steer = avg_vel - parent_vel.xy();
        let (mut parent_steer, _, _) = q_boids.get_mut(parent_ent).unwrap();
        parent_steer.0 += to_avg_vel_steer.normalize() * config.alignment_strength;
    }
}

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

pub struct BoidsPlaygroundPlugin {
    pub x_count: i32,
    pub y_count: i32,
    pub x_gap: f32,
    pub y_gap: f32,
}

#[derive(Resource, Debug)]
struct BoidPlacement {
    x_gap: f32,
    y_gap: f32,
    x_count: i32,
    y_count: i32,
}

fn setup(mut commands: Commands, boid_placement: Res<BoidPlacement>) {
    commands.spawn(Camera2d);

    let BoidPlacement {
        x_gap,
        y_gap,
        x_count,
        y_count,
    } = *boid_placement;

    for x in 0..x_count {
        for y in 0..y_count {
            let loc = (x as f32 * x_gap, y as f32 * y_gap).into();
            let angle = rand::thread_rng().gen_range((0.)..std::f32::consts::TAU);
            let trigger = BoidSpawn {
                loc,
                angle,
                special: x == 0 && y == 0,
            };
            commands.trigger(trigger);
        }
    }
}

impl Plugin for BoidsPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        let BoidsPlaygroundPlugin {
            x_count,
            y_count,
            x_gap,
            y_gap,
        } = *self;
        app.insert_resource(BoidPlacement {
            x_gap,
            y_gap,
            x_count,
            y_count,
        })
        .add_systems(Startup, setup);
    }
}
