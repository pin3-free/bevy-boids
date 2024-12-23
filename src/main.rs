use avian2d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
#[require(Collider)]
pub struct BoidCollider;

#[derive(Component)]
#[require(Collider, Sensor, CollidingEntities)]
pub struct BoidVisionCone;

#[derive(Component)]
#[require(Transform, ViewVisibility, Visibility, Mesh2d, MeshMaterial2d<ColorMaterial>, RigidBody)]
pub struct Boid;

#[derive(Event, Default)]
pub struct BoidSpawn {
    loc: Vec2,
    angle: f32,
    special: bool,
}

#[derive(Component)]
pub struct SpecialBoid;

fn spawn_boid(
    trigger: Trigger<BoidSpawn>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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

    let boid = commands
        .spawn((
            Boid,
            Transform::from_translation(trigger.loc.extend(0.))
                .with_rotation(Quat::from_rotation_z(trigger.angle)),
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
            let vision_scale = 80.;
            let collider_triangle = Collider::triangle(
                (0., 0.).into(),
                (-vision_scale / 1.5, vision_scale).into(),
                (vision_scale / 1.5, vision_scale).into(),
            );
            parent.spawn((
                Name::new("Vision Cone"),
                BoidVisionCone,
                Collider::compound(vec![
                    (Vec2::new(0., 0.), 0., collider_triangle.clone()),
                    (
                        Vec2::new(0., 0.),
                        std::f32::consts::FRAC_PI_3,
                        collider_triangle.clone(),
                    ),
                    (
                        Vec2::new(0., 0.),
                        -std::f32::consts::FRAC_PI_3,
                        collider_triangle.clone(),
                    ),
                ]),
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

fn move_boids_forward(mut q_boids: Populated<(&mut LinearVelocity, &Transform), With<Boid>>) {
    let speed = 150.;
    for (mut linear_velocity, transform) in q_boids.iter_mut() {
        let rotated = transform.rotation.mul_vec3(Vec2::Y.extend(0.)).truncate();
        linear_velocity.x = rotated.x * speed;
        linear_velocity.y = rotated.y * speed;
    }
}

fn screenwrap_boids(
    mut q_boids: Populated<(&mut Transform), With<Boid>>,
    q_window: Populated<&Window>,
) {
    let window = q_window.single();
    let window_size = window.size();
    let world_halfwidth = window_size.x / 2.;
    let world_halfheight = window_size.y / 2.;
    let make_wrap = move |val: f32| {
        let res = move |mut num| {
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

    for (mut transform) in q_boids.iter_mut() {
        transform.translation.x = wrapx(transform.translation.x);
        transform.translation.y = wrapy(transform.translation.y);
    }
}

#[derive(PhysicsLayer, Default)]
enum GameCollisionLayer {
    #[default]
    Default,
    VisionCones,
    Boids,
}

fn boid_avoidance(
    mut q_boid_velocities: Populated<&mut AngularVelocity, With<Boid>>,
    q_boid_transforms: Populated<&Transform, With<Boid>>,
    // q_names: Populated<&Name>,
    q_vision_cones: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
) {
    let avoidance_speed = 100.;
    for (colliding_entities, parent) in q_vision_cones.iter() {
        let Ok((parent_tr)) = q_boid_transforms.get(parent.get()) else {
            error!("Vision cone with no parent??");
            continue;
        };
        let colliding_transforms = colliding_entities
            .iter()
            .map(|e| {
                let tr = q_boid_transforms
                    .get(*e)
                    .expect("Colliding boid should have a transform");
                tr
            })
            .collect::<Vec<_>>();
        let parent_forward = parent_tr.rotation.mul_vec3(Vec3::Y).truncate();
        let parent_right = parent_tr.rotation.mul_vec3(Vec3::X).truncate();
        let mut angular_vel = q_boid_velocities
            .get_mut(parent.get())
            .expect("Parent boid should have angular velocity");
        let mut total_rotation = 0.;
        for other_tr in colliding_transforms {
            let direction = (other_tr.translation - parent_tr.translation).truncate();
            let rotation_coef = direction.normalize().dot(parent_forward.normalize()).abs();
            let relative_dir = direction.project_onto(parent_right).normalize();
            // Positive AngularVelocity is countercockwise
            let delta = avoidance_speed / (direction.length());
            if relative_dir == parent_right {
                warn!("Steering clockwise");
                total_rotation -= delta;
            } else {
                warn!("Steering counterclockwise");
                total_rotation += delta;
            }
        }
        angular_vel.0 = total_rotation;
    }
}

fn debug_special_boid(
    q_special: Populated<Entity, With<SpecialBoid>>,
    q_vision_cones: Populated<(&CollidingEntities, &Parent), With<BoidVisionCone>>,
) {
    for (colliding_entities, parent) in q_vision_cones.iter() {
        if q_special.get(parent.get()).is_ok() {
            info!("Special boid sees {} boids", colliding_entities.len());
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let x_gap = 20.;
    let y_gap = 20.;
    let x_count = 2;
    let y_count = 2;

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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_boids_forward, boid_avoidance, screenwrap_boids).chain(),
        )
        // .add_systems(Update, debug_special_boid)
        .add_event::<BoidSpawn>()
        .add_observer(spawn_boid)
        .run();
}
