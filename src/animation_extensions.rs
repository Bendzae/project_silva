use bevy::{ecs::system::Resource, prelude::*};
// use bevy_inspector_egui::Inspectable;
use std::{f32::consts::E, ops::Deref};

use crate::PlayerState;

#[derive(Component)]
pub struct CrossFadePlayer {
    from: Option<Handle<AnimationClip>>,
    to: Option<Handle<AnimationClip>>,
    transition_time: f32,
    elapsed: f32,
}

impl Default for CrossFadePlayer {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            transition_time: 0.0,
            elapsed: 0.0,
        }
    }
}

impl CrossFadePlayer {
    pub fn crossfade(
        &mut self,
        from: Handle<AnimationClip>,
        to: Handle<AnimationClip>,
        transition_time: f32,
    ) -> &mut Self {
        *self = Self {
            from: Some(from),
            to: Some(to),
            transition_time,
            ..Default::default()
        };
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }
}

fn init_crossfade_player_system(
    animation_players: Query<Entity, With<AnimationPlayer>>,
    mut commands: Commands,
    mut did_run: Local<bool>,
) {
    if *did_run {
        return;
    }

    for e in animation_players.iter() {
        commands.entity(e).insert(CrossFadePlayer::default());
        *did_run = true;
    }
}

fn crossfade_player_system(
    time: Res<Time>,
    animations: Res<Assets<AnimationClip>>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer, &mut CrossFadePlayer)>,
    names: Query<&Name>,
    mut transforms: Query<&mut Transform>,
    children: Query<&Children>,
) {
    for (entity, mut animation_player, mut crossfade_player) in animation_players.iter_mut() {
        if crossfade_player.from.is_none() || crossfade_player.to.is_none() {
            continue;
        }

        if let Some(from_clip) = animations.get(crossfade_player.from.as_ref().unwrap()) {
            if let Some(to_clip) = animations.get(crossfade_player.to.as_ref().unwrap()) {
                // Stop any other animation
                animation_player.pause();
                // Get elapsed time of old animation
                let old_elapsed = animation_player.elapsed();
                // Forward time
                crossfade_player.elapsed += time.delta_seconds();
                let mut elapsed = crossfade_player.elapsed;

                let mut fade_factor = elapsed / crossfade_player.transition_time;

                if fade_factor >= 1.0 {
                    fade_factor = 1.0; // set to exactly one so the last step of the interpolation is exact

                    let next_animation = crossfade_player.to.as_ref().unwrap().clone_weak();

                    crossfade_player.reset();

                    animation_player.play(next_animation).repeat();
                }

                'entity: for (path, from_curves) in from_clip.curves() {
                    let to_curves = to_clip.curves().get(path);

                    // PERF: finding the target entity can be optimised
                    let mut current_entity = entity;
                    // Ignore the first name, it is the root node which we already have
                    for part in path.parts.iter().skip(1) {
                        let mut found = false;
                        if let Ok(children) = children.get(current_entity) {
                            for child in children.deref() {
                                if let Ok(name) = names.get(*child) {
                                    if name == part {
                                        // Found a children with the right name, continue to the next part
                                        current_entity = *child;
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !found {
                            warn!("Entity not found for path {:?} on part {:?}", path, part);
                            continue 'entity;
                        }
                    }
                    if let Ok(mut transform) = transforms.get_mut(current_entity) {
                        let mut to_rotation: Option<Quat> = None;
                        let mut to_translation: Option<Vec3> = None;
                        let mut to_scale: Option<Vec3> = None;

                        // to_curves
                        if let Some(to_curves) = to_curves {
                            for curve in to_curves {
                                match &curve.keyframes {
                                    Keyframes::Rotation(keyframes) => {
                                        to_rotation = Some(keyframes[0]);
                                    }
                                    Keyframes::Translation(keyframes) => {
                                        to_translation = Some(keyframes[0]);
                                    }
                                    Keyframes::Scale(keyframes) => {
                                        to_scale = Some(keyframes[0]);
                                    }
                                }
                            }
                        }
                        // from_curves
                        for curve in from_curves {
                            // Find the current keyframe
                            // PERF: finding the current keyframe can be optimised
                            let stoped_keyframe = match curve
                                .keyframe_timestamps
                                .binary_search_by(|probe| probe.partial_cmp(&old_elapsed).unwrap())
                            {
                                Ok(i) => i,
                                Err(0) => continue, // this curve isn't started yet
                                Err(n) if n > curve.keyframe_timestamps.len() - 1 => continue, // this curve is finished
                                Err(i) => i - 1,
                            };

                            match &curve.keyframes {
                                Keyframes::Rotation(keyframes) => {
                                    if let Some(to) = to_rotation {
                                        transform.rotation = keyframes[stoped_keyframe].slerp(to, fade_factor)
                                    } else {
                                        transform.rotation = keyframes[stoped_keyframe];
                                    }
                                }
                                Keyframes::Translation(keyframes) => {
                                    if let Some(to) = to_translation {
                                        transform.translation = keyframes[stoped_keyframe].lerp(to, fade_factor)
                                    } else {
                                        transform.translation = keyframes[stoped_keyframe];
                                    }
                                }
                                Keyframes::Scale(keyframes) => {
                                    if let Some(to) = to_scale {
                                        transform.scale = keyframes[stoped_keyframe].lerp(to, fade_factor)
                                    } else {
                                        transform.scale = keyframes[stoped_keyframe];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct AnimationExtensionsPlugin;

impl Plugin for AnimationExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(init_crossfade_player_system)
            .add_system_to_stage(
                CoreStage::PostUpdate,
                crossfade_player_system
                    .before(bevy::transform::TransformSystem::TransformPropagate)
                    .after(bevy::animation::animation_player),
            );
    }
}
