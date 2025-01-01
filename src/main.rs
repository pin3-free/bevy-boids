use boids::{BoidsPlugin, SpawnBoid};
use editor::EditorPlugin;
use input::SimulationInputPlugin;

use crate::prelude::*;

mod prelude;

mod editor;

mod input;

mod boids;

#[cfg(test)]
mod tests;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct DesiredDir(Vec2);

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), MainCamera));

    let x_count = 10;
    let y_count = 10;
    let x_gap = 50.;
    let y_gap = 50.;

    for x in 0..x_count {
        for y in 0..y_count {
            let loc = (x as f32 * x_gap, y as f32 * y_gap).into();
            let angle = rand::thread_rng().gen_range((0.)..std::f32::consts::TAU);
            // let angle = -std::f32::consts::PI * x as f32 + std::f32::consts::FRAC_PI_2;
            let trigger = SpawnBoid {
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
            EditorPlugin,
            SimulationInputPlugin,
            // WorldInspectorPlugin::new(),
            // PhysicsDebugPlugin::default(),
            BoidsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}
