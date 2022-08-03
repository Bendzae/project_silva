use std::time::Duration;

use bevy::prelude::*;

use crate::player::{PlayerState, PlayerStateEnum};

pub struct Animations(pub Vec<Handle<AnimationClip>>);

pub fn player_animation_system(
    animations: Res<Animations>,
    mut player_query: Query<(&Parent, &mut AnimationPlayer)>,
    mut state_query: Query<&mut PlayerState>,
) {
    let idle_index = 0;
    let run_index = 1;

    for (parent, mut player) in player_query.iter_mut() {
        if let Ok(mut state) = state_query.get_single_mut() {
            match state.state {
                PlayerStateEnum::IDLE => {
                    if state.animation.is_none() || state.animation.unwrap() != idle_index {
                        if state.animation.is_none() {
                            player.play(animations.0[idle_index].clone_weak()).repeat();
                        } else {
                            player
                                .cross_fade(
                                    animations.0[idle_index].clone_weak(),
                                    Duration::from_secs_f32(0.3),
                                )
                                .repeat();
                        }
                        state.animation = Some(idle_index);
                    }
                }
                PlayerStateEnum::MOVING => {
                    if state.animation.is_none() || state.animation.unwrap() != run_index {
                        player
                            .cross_fade(
                                animations.0[run_index].clone_weak(),
                                Duration::from_secs_f32(0.25),
                            )
                            .set_speed(1.3)
                            .repeat();
                        state.animation = Some(run_index);
                    }
                }
            };
        }
    }
}
