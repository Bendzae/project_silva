use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use std::ops::Deref;

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

fn crossfade_player_system(
    time: Res<Time>,
    animations: Res<Assets<AnimationClip>>,
    mut crossfade_players: Query<(&mut CrossFadePlayer, &Children)>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    names: Query<&Name>,
    mut transforms: Query<&mut Transform>,
    children: Query<&Children>,
) {
    for (mut crossfade_player, crossfade_children) in crossfade_players.iter_mut() {
        if crossfade_player.from.is_none() || crossfade_player.to.is_none() {
            continue;
        }

        // let mut res = None;
        // for &child in crossfade_children.iter() {
        //     res = match animation_players.get(child) {
        //         Ok(_) => Some(child),
        //         Err(_) => continue,
        //     };
        //     break;
        // }

        // let ani_player_entity = match res {
        //     Some(r) => r,
        //     None => panic!("Not AnimationPlayer found!"),
        // };

        // let mut animation_player = animation_players.get_mut(ani_player_entity).unwrap();

        let (ani_player_entity, mut animation_player) = animation_players.get_single_mut().unwrap();

        if let Some(from_clip) = animations.get(crossfade_player.from.as_ref().unwrap()) {
            if let Some(to_clip) = animations.get(crossfade_player.to.as_ref().unwrap()) {
                // Forward time
                crossfade_player.elapsed += time.delta_seconds();
                let mut elapsed = crossfade_player.elapsed;

                let fade_factor = elapsed / crossfade_player.transition_time;

                if elapsed >= crossfade_player.transition_time || fade_factor > 1.0 {
                    animation_player
                        .play(crossfade_player.to.as_ref().unwrap().clone_weak())
                        .repeat();
                    crossfade_player.reset();
                    continue;
                }

                'entity: for (path, from_curves) in from_clip.curves() {
                    let to_curves = to_clip.curves().get(path);

                    // PERF: finding the target entity can be optimised
                    let mut current_entity = ani_player_entity;
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
                        animation_player.pause();
                        for curve in from_curves {
                            match &curve.keyframes {
                                Keyframes::Rotation(keyframes) => {
                                    if let Some(to) = to_rotation {
                                        transform.rotation = keyframes[0].slerp(to, fade_factor)
                                    } else {
                                        transform.rotation = keyframes[0];
                                    }
                                }
                                Keyframes::Translation(keyframes) => {
                                    if let Some(to) = to_translation {
                                        transform.translation = keyframes[0].lerp(to, fade_factor)
                                    } else {
                                        transform.translation = keyframes[0];
                                    }
                                }
                                Keyframes::Scale(keyframes) => {
                                    if let Some(to) = to_scale {
                                        transform.scale = keyframes[0].lerp(to, fade_factor)
                                    } else {
                                        transform.scale = keyframes[0];
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
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            crossfade_player_system
                .before(bevy::transform::TransformSystem::TransformPropagate)
                .after(bevy::hierarchy::HierarchySystem::ParentUpdate),
        );
    }
}
