use bevy::prelude::*;

use crate::movement::{MovementSpeed, MovementTarget};

#[derive(Component)]
pub struct Player;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlayerStateEnum {
    IDLE,
    MOVING,
}

#[derive(Component)]
pub struct PlayerState {
    pub state: PlayerStateEnum,
    pub animation: Option<usize>,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub _p: Player,
    pub name: crate::NameV2,
    pub movement_speed: MovementSpeed,
    pub state: PlayerState,
    pub movement_target: MovementTarget,
    #[bundle]
    pub scene_bundle: SceneBundle,
}

impl Default for PlayerBundle {
    fn default() -> PlayerBundle {
        return PlayerBundle {
            _p: Player,
            name: crate::NameV2("unknown".to_string()),
            movement_speed: MovementSpeed(3.0),
            state: PlayerState {
                state: PlayerStateEnum::IDLE,
                animation: None,
            },
            movement_target: MovementTarget::default(),
            scene_bundle: SceneBundle::default(),
        };
    }
}
