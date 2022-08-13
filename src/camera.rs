use bevy::{math::vec3, prelude::*};

use crate::{input::ZoomEvent, Player};

const ZOOM_FACTOR: f32 = 10.0;

pub fn camera_follow_player_system(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut inital_position: Local<Option<Vec3>>,
    mut current_zoom: Local<f32>,
    mut zoom_events: EventReader<ZoomEvent>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        if inital_position.is_none() {
            *inital_position = Some(camera_transform.translation);
            *current_zoom = 0.0;
        }

        for e in zoom_events.iter() {
            let tmp = *current_zoom;
            *current_zoom = (tmp + e.0 * time.delta_seconds() * ZOOM_FACTOR).clamp(-30.0, 10.0);
        }

        if let Ok(player_transform) = player_query.get_single() {
            let pt = player_transform.translation;
            let forward = camera_transform.forward();
            let target_pos =
                (*inital_position).unwrap() + Vec3::new(pt.x, 0.0, pt.z) + forward * *current_zoom;
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, 7.0 * time.delta_seconds())
        }
    }
}