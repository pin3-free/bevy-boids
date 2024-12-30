use crate::{
    boids::targets::{SeekTarget, SpawnTarget},
    prelude::*,
    MainCamera,
};

pub struct SimulationInputPlugin;

impl Plugin for SimulationInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_mouse_inputs);
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

pub fn move_drag(trigger: Trigger<Pointer<Drag>>, mut q_targets: Query<&mut Transform>) {
    let ent = trigger.entity();
    let mut transform = q_targets.get_mut(ent).expect("Target should be present");
    transform.translation += Vec3::new(trigger.delta.x, -trigger.delta.y, 0.);
}
