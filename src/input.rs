use bevy::input::mouse::MouseMotion;
use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

use crate::debug::TestDebugComponent;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InputCommand {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    ACTION,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct InputEvent(pub InputCommand);

pub fn input_system(input: Res<Input<KeyCode>>, mut event: EventWriter<InputEvent>) {
    if input.pressed(KeyCode::Left) {
        event.send(InputEvent(InputCommand::LEFT));
    }
    if input.pressed(KeyCode::Right) {
        event.send(InputEvent(InputCommand::RIGHT));
    }
    if input.pressed(KeyCode::Up) {
        event.send(InputEvent(InputCommand::UP));
    }
    if input.pressed(KeyCode::Down) {
        event.send(InputEvent(InputCommand::DOWN));
    }
}

#[derive(Clone, Copy)]
pub struct MouseFloorPosition(pub Vec3);

fn my_cursor_system(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut test: Query<&mut Transform, With<TestDebugComponent>>,
    mut event: EventWriter<MouseFloorPosition>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let RenderTarget::Window(id) = camera.target {
        match wnds.get(id) {
            Some(wnd) => wnd,
            None => return,
        }
    } else {
        match wnds.get_primary() {
            Some(wnd) => wnd,
            None => return,
        }
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(1.0));

        // construct ray from screenpoint in view direction
        let ray_dir = (world_pos - camera_transform.translation()).normalize();

        // find intersection length with y-0-plane
        let ray_length = (-world_pos.y) / ray_dir.y;

        // find postion on plane
        let zero_plane_pos = world_pos + ray_length * ray_dir;

        event.send(MouseFloorPosition(zero_plane_pos));

        if let Ok(mut test_transform) = test.get_single_mut() {
            test_transform.translation = zero_plane_pos;
        }
    }
}

pub fn debug_input_system(time: Res<Time>, mut events: EventReader<InputEvent>) {
    for event in events.iter() {
        println!(
            "Event: received: {:?} at {}",
            event.0,
            time.seconds_since_startup()
        );
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>()
            .add_event::<MouseFloorPosition>()
            .add_system(input_system.after(InputSystem))
            .add_system(my_cursor_system);
        // .add_system(debug_input_system.after(input_system));
    }

    fn name(&self) -> &str {
        "InputPlugin"
    }
}
