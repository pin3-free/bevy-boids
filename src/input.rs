use bevy::input::{
    gestures::{DoubleTapGesture, PanGesture, PinchGesture, RotationGesture},
    mouse::MouseWheel,
};
use bevy_inspector_egui::{quick::ResourceInspectorPlugin, InspectorOptions};

use crate::{
    boids::targets::{SeekTarget, SpawnTarget},
    prelude::*,
    MainCamera,
};

pub struct SimulationInputPlugin;

impl Plugin for SimulationInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_mouse_inputs, handle_scrolling, handle_gestures),
        )
        .insert_resource(InputConfig::default())
        .register_type::<InputConfig>()
        .add_plugins(ResourceInspectorPlugin::<InputConfig>::default());
    }
}

#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource)]
pub struct InputConfig {
    scroll_speed: f32,
    zoom_speed: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_speed: 1.,
            zoom_speed: 1.,
        }
    }
}

fn handle_mouse_inputs(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboad: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    q_camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
) {
    let (camera, camera_global_tr) = *q_camera;
    if mouse.just_pressed(MouseButton::Left) && keyboad.pressed(KeyCode::ShiftLeft) {
        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };
        let world_pos = camera
            .viewport_to_world_2d(camera_global_tr, cursor_pos)
            .expect("Cursor should convert");
        commands.trigger(SpawnTarget::<SeekTarget>::new(world_pos));
    }
}

// Getsures for MacOS
fn handle_gestures(
    mut evr_gesture_pinch: EventReader<PinchGesture>,
    q_camera: Single<&mut OrthographicProjection, With<MainCamera>>,
    config: Res<InputConfig>,
) {
    let mut proj = q_camera;
    for ev_pinch in evr_gesture_pinch.read() {
        // Positive numbers are zooming in
        // Negative numbers are zooming out
        println!("Two-finger zoom by {}", ev_pinch.0);
        let new_scale = proj.scale - ev_pinch.0 * config.zoom_speed;
        proj.scale = new_scale.clamp(1., 4.);
    }
}

fn handle_scrolling(
    mut scroll: EventReader<MouseWheel>,
    q_camera: Single<&mut Transform, With<MainCamera>>,
    config: Res<InputConfig>,
) {
    let mut camera_tr = q_camera;
    for ev in scroll.read() {
        match ev.unit {
            // Line is for mice and wheels
            bevy::input::mouse::MouseScrollUnit::Line => {
                println!(
                    "Scroll (line units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
            }
            // Pixel is for touchpads
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                println!(
                    "Scroll (pixel units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );

                camera_tr.translation.x += -ev.x * config.scroll_speed;
                camera_tr.translation.y += ev.y * config.scroll_speed;
            }
        }
    }
}

pub fn move_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut q_targets: Query<&mut Transform>,
    q_camera: Single<&OrthographicProjection, With<MainCamera>>,
) {
    let ent = trigger.entity();
    let mut transform = q_targets.get_mut(ent).expect("Target should be present");
    transform.translation += Vec3::new(
        trigger.delta.x * q_camera.scale,
        -trigger.delta.y * q_camera.scale,
        0.,
    );
}
