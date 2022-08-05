use bevy::prelude::*;

use crate::{input::{InputEvent, MouseFloorPosition, InputCommand}, player::{PlayerState, Player, PlayerStateEnum}};

#[derive(Component)]
pub struct MovementSpeed(pub f32);


#[derive(Component)]
pub struct MovementTarget {
    pub current_target: Option<Vec3>,
}

impl Default for MovementTarget {
    fn default() -> Self {
        return Self {
            current_target: Some(Vec3::ZERO),
        };
    }
}

pub fn player_movement_system(
    mut input_event: EventReader<InputEvent>,
    mut mouse_event: EventReader<MouseFloorPosition>,
    mut mouse_button_event: Res<Input<MouseButton>>,
    mut query: Query<
        (
            &mut Transform,
            &MovementSpeed,
            &mut PlayerState,
            &mut MovementTarget,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut target_rot: Local<Quat>,
) {
    let turn_speed: f32 = 15.0;
    for (mut transform, speed, mut state, mut target) in query.iter_mut() {
        let mut direction = Vec3::default();
        for event in input_event.iter() {
            match event.0 {
                InputCommand::RIGHT => direction.x += 1.0,
                InputCommand::LEFT => direction.x -= 1.0,
                InputCommand::UP => direction.z -= 1.0,
                InputCommand::DOWN => direction.z += 1.0,
                InputCommand::ACTION => (),
            }
        }
        if direction.length() > 0.0 {
            target.current_target = None;
        }

        if mouse_button_event.pressed(MouseButton::Right) {
            for event in mouse_event.iter() {
                target.current_target = Some(event.0);
            }
        }
        if let Some(current_target) = target.current_target {
            direction = current_target - transform.translation;
        }

        if direction.length() > time.delta_seconds() * speed.0 {
            let normalized_dir = direction.normalize();
            transform.translation += normalized_dir * speed.0 * time.delta_seconds();

            // Rotation
            let angle = normalized_dir.angle_between(Vec3::new(0.0, 0.0, 1.0));
            *target_rot = Quat::from_rotation_y(if normalized_dir.x > 0.0 {
                angle
            } else {
                -angle
            });
            state.state = PlayerStateEnum::MOVING;
        } else {
            if let Some(current_target) = target.current_target {
                transform.translation = current_target;
            }
            state.state = PlayerStateEnum::IDLE;
        }

        let angle_to_target = transform.rotation.angle_between(*target_rot);
        if angle_to_target > 0.0 {
            let t = turn_speed / angle_to_target;
            transform.rotation = transform
                .rotation
                .slerp(*target_rot, 1.0_f32.min(t * time.delta_seconds()));
        }
    }
}