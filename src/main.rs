use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use boids::{obstacles::SpawnObstacle, BoidsPlugin, SpawnBoid};
use i_cant_believe_its_not_bsn::*;
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

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d::default(), MainCamera));

    let x_count = 15;
    let y_count = 15;
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

    let bounding_box_color = Color::srgb(0.1, 0.1, 0.1);
    commands.trigger(SpawnObstacle::rectangle(2400., 1800.).with_color(bounding_box_color));
}

#[derive(Component)]
pub struct FpsCounter;

fn fps_counter(fps: f64) -> Template {
    let temp = template! {
        {( Text::new(""), TextFont::from_font_size(28.) )} [
            {TextSpan::new("FPS: ")};
            {TextSpan::new(format!("{:>4.0}", fps))};
        ];
    };
    temp
}

fn fps_system(mut commands: Commands, diagnostics: Res<DiagnosticsStore>) {
    let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    else {
        return;
    };

    let template = template!({
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.),
            left: Val::Px(5.),
            ..Default::default()
        }
    } [
            @{ fps_counter(value) };
        ];
    );

    commands.build(template);
}

#[derive(Component)]
pub struct FpsRoot;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            FrameTimeDiagnosticsPlugin::default(),
            // EditorPlugin,
            SimulationInputPlugin,
            // WorldInspectorPlugin::new(),
            // PhysicsDebugPlugin::default(),
            BoidsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, fps_system)
        .run();
}
