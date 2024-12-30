use crate::{input::move_drag, prelude::*};

use super::{BoidVisionQuery, BoidsQuery, SimulationConfig};

pub struct SeekPlugin;

impl Plugin for SeekPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (seek_behaviour, despawn_seek_targets))
            .add_plugins(MeshPickingPlugin::default())
            .add_event::<SpawnSeekTarget>()
            .add_observer(spawn_seek_target);
    }
}

#[derive(Event)]
pub struct SpawnSeekTarget {
    pub pos: Vec2,
}

fn despawn_seek_targets(
    mut commands: Commands,
    q_targets: Query<(Entity, &CollidingEntities), (With<SeekTarget>, Changed<CollidingEntities>)>,
    q_boids: Populated<BoidsQuery>,
) {
    for (ent, colliding) in q_targets.iter() {
        if colliding.is_empty() {
            continue;
        }

        if let Some(_colliding_boid) = colliding.iter().find(|e| q_boids.get(**e).is_ok()) {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn spawn_seek_target(
    trigger: Trigger<SpawnSeekTarget>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let target_mesh = Circle::new(10.);
    let target_material = ColorMaterial::from_color(Color::srgb(0., 1., 0.));

    commands
        .spawn((
            Transform::from_translation(trigger.pos.extend(0.)),
            Mesh2d(meshes.add(target_mesh)),
            MeshMaterial2d(materials.add(target_material)),
            crate::boids::SeekTarget,
            Collider::circle(10.),
            CollisionLayers::new(
                GameCollisionLayer::Targets,
                [GameCollisionLayer::VisionCones, GameCollisionLayer::Boids],
            ),
            CollidingEntities::default(),
        ))
        .observe(move_drag);
}

#[derive(Component)]
pub struct Seek;

#[derive(Component)]
pub struct SeekTarget;

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
