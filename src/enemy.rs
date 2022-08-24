use bevy::prelude::*;

use crate::{movement::MovementSpeed, NameV2};

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub _e: Enemy,
    pub name: NameV2,
    pub movement_speed: MovementSpeed,
    #[bundle]
    pub scene_bundle: SceneBundle,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        return EnemyBundle {
            _e: Enemy,
            name: NameV2("unknown".to_string()),
            movement_speed: MovementSpeed(1.0),
            scene_bundle: SceneBundle::default(),
        };
    }
}
