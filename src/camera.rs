use bevy::prelude::*;

use crate::Player;

pub fn camera_follow_player_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut inital_position: Local<Option<Vec3>>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if inital_position.is_none() {
            *inital_position = Some(camera_transform.translation);
        }
        if let Ok(player_transform) = player_query.get_single() {
            let pt = player_transform.translation;
            let target_pos = (*inital_position).unwrap() + Vec3::new(pt.x, 0.0, pt.z);
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, 7.0 * time.delta_seconds())
        }
    }
}