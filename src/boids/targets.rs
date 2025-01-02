use crate::{input::move_drag, prelude::*};
use std::marker::PhantomData;

use super::BoidsQuery;

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTarget<SeekTarget>>()
            .add_event::<SpawnTarget<FleeTarget>>()
            .add_observer(spawn_target::<SeekTarget>)
            .add_observer(spawn_target::<FleeTarget>)
            .add_systems(FixedUpdate, despawn_targets::<FleeTarget>);
    }
}

#[derive(Component, Default)]
pub struct SeekTarget;

#[derive(Component, Default)]
pub struct FleeTarget;

#[derive(Component, Default)]
pub struct Obstacle;

#[derive(Component, Default)]
pub struct Target<T> {
    marker: PhantomData<T>,
}

#[derive(Event, Default)]
pub struct SpawnTarget<T> {
    pos: Vec2,
    marker: PhantomData<T>,
}

impl<T> SpawnTarget<T>
where
    T: Default,
{
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }
}

fn spawn_target<T>(
    trigger: Trigger<SpawnTarget<T>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) where
    T: Component + Default,
    Target<T>: BoidTarget,
{
    let target_mesh = Target::<T>::mesh();
    let target_material = ColorMaterial::from_color(Target::<T>::color());

    commands
        .spawn((
            Transform::from_translation(trigger.pos.extend(1.)),
            Mesh2d(meshes.add(target_mesh)),
            MeshMaterial2d(materials.add(target_material)),
            T::default(),
            Target::<T>::default(),
            Target::<T>::collider(),
            Target::<T>::collision_layers(),
            CollidingEntities::default(),
        ))
        .observe(move_drag);
}

fn despawn_targets<T>(
    mut commands: Commands,
    q_targets: Query<(Entity, &CollidingEntities), (With<Target<T>>, Changed<CollidingEntities>)>,
    q_boids: Populated<BoidsQuery>,
) where
    Target<T>: Component,
{
    for (ent, colliding) in q_targets.iter() {
        if colliding.is_empty() {
            continue;
        }

        if let Some(_colliding_boid) = colliding.iter().find(|e| q_boids.get(**e).is_ok()) {
            commands.entity(ent).despawn_recursive();
        }
    }
}

trait BoidTarget {
    fn color() -> Color;

    fn mesh() -> Circle {
        Circle::new(Self::radius())
    }

    fn collider() -> Collider {
        Collider::circle(Self::radius())
    }

    fn collision_layers() -> CollisionLayers {
        CollisionLayers::new(
            GameCollisionLayer::Targets,
            [GameCollisionLayer::VisionCones, GameCollisionLayer::Boids],
        )
    }

    fn radius() -> f32 {
        10.
    }
}

impl BoidTarget for Target<SeekTarget> {
    fn color() -> Color {
        Color::srgb(0., 1., 0.)
    }
}

impl BoidTarget for Target<FleeTarget> {
    fn color() -> Color {
        Color::srgb(1., 0., 0.)
    }
}
