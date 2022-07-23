use bevy::utils::Duration;
use bevy::{input::mouse::MouseButtonInput, prelude::*};
use input::MouseFloorPosition;
// use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use std::f32::consts::PI;

use crate::input::{input_system, InputCommand, InputEvent, InputPlugin};

mod input;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct MovementSpeed(f32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlayerStateEnum {
    IDLE,
    MOVING,
}

#[derive(Component)]
struct PlayerState {
    state: PlayerStateEnum,
    animation: Option<usize>,
}

#[derive(Component)]
struct MovementTarget {
    current_target: Option<Vec3>,
}

impl Default for MovementTarget {
    fn default() -> Self {
        return Self {
            current_target: Some(Vec3::ZERO),
        };
    }
}

#[derive(Bundle)]
struct PlayerBundle {
    _p: Player,
    name: Name,
    movement_speed: MovementSpeed,
    state: PlayerState,
    movement_target: MovementTarget,
    #[bundle]
    scene_bundle: SceneBundle,
}

impl Default for PlayerBundle {
    fn default() -> PlayerBundle {
        return PlayerBundle {
            _p: Player,
            name: Name("unknown".to_string()),
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

#[derive(Component)]
struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    _e: Enemy,
    name: Name,
    transform: Transform,
    movement_speed: MovementSpeed,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        return EnemyBundle {
            _e: Enemy,
            name: Name("unknown".to_string()),
            transform: Transform::default(),
            movement_speed: MovementSpeed(1.0),
        };
    }
}

#[derive(Component)]
struct TestDebugComponent;

#[derive(Bundle)]
pub struct TestBundle {
    _t: TestDebugComponent,
    #[bundle]
    pbr_bundle: PbrBundle,
}

impl Default for TestBundle {
    fn default() -> TestBundle {
        return TestBundle {
            _t: TestDebugComponent,
            pbr_bundle: PbrBundle::default(),
        };
    }
}

struct Animations(Vec<Handle<AnimationClip>>);

fn spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    // Insert a resource with the current scene information
    commands.insert_resource(Animations(vec![
        asset_server.load("silva_main_char.glb#Animation0"),
        asset_server.load("silva_main_char.glb#Animation1"),
    ]));

    // Floor
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        ..default()
    });

    // Box
    commands.spawn_bundle(TestBundle {
        pbr_bundle: PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
            material: materials.add(Color::rgb(0.2, 0.0, 0.0).into()),
            ..default()
        },
        ..Default::default()
    });

    // Player
    commands.spawn_bundle(PlayerBundle {
        name: Name("Player_1".to_string()),
        scene_bundle: SceneBundle {
            scene: asset_server.load("silva_main_char.glb#Scene0"),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        ..PlayerBundle::default()
    });
    // .with_children(|parent| {
    //     parent.spawn_scene(asset_server.load("silva_main_char.glb#Scene0"));
    // });

    // Enemies
    for i in 0..3 {
        commands.spawn_bundle(EnemyBundle {
            name: Name(format!("enemy_{i}")),
            ..EnemyBundle::default()
        });
    }

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });

    // directional light
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            ..default()
        },
        transform: Transform::from_xyz(0.0, 8.0, 0.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -(PI / 4.0),
            PI / 8.0,
            0.0,
        )),
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn player_movement_system(
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

fn player_animation_system(
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

fn camera_follow_player_system(
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

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // .add_plugins_with(DefaultPlugins, |plugins| {
        //     plugins.disable::<AnimationPlugin>()
        // })
        .add_plugin(InputPlugin)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_system)
        // .add_startup_system_to_stage(StartupStage::PostStartup, debug_spawn_system)
        .add_system(player_movement_system.after(input_system))
        .add_system(player_animation_system.after(player_movement_system))
        .add_system(camera_follow_player_system)
        .run();
}
