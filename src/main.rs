use bevy::utils::tracing::Event;
use bevy::{ecs::bundle, prelude::*};
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

#[derive(Bundle)]
struct PlayerBundle {
    _p: Player,
    name: Name,
    movement_speed: MovementSpeed,
    state: PlayerState,
    #[bundle]
    transform_bundle: TransformBundle,
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
            transform_bundle: TransformBundle::default(),
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

struct Animations(Vec<Handle<AnimationClip>>);

fn spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    scene_spawner: ResMut<SceneSpawner>,
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

    // Player
    commands
        .spawn_bundle(PlayerBundle {
            name: Name("Player_1".to_string()),
            transform_bundle: TransformBundle {
                local: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(1.0, 1.0, 1.0),
                    ..default()
                },
                ..default()
            },
            ..PlayerBundle::default()
        })
        .with_children(|cell| {
            cell.spawn_scene(asset_server.load("silva_main_char.glb#Scene0"));
        });

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
            (PI / 8.0),
            0.0,
        )),
        ..default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn player_movement_system(
    mut input_event: EventReader<InputEvent>,
    mut query: Query<(&mut Transform, &MovementSpeed, &mut PlayerState), With<Player>>,
    time: Res<Time>,
    mut target_rot: Local<Quat>,
) {
    let turn_speed: f32 = 20.0;
    for (mut transform, speed, mut state) in query.iter_mut() {
        let mut direction = Vec3::default();
        for event in input_event.iter() {
            match event.0 {
                InputCommand::RIGHT => direction.x += 1.0,
                InputCommand::LEFT => direction.x -= 1.0,
                InputCommand::UP => direction.z -= 1.0,
                InputCommand::DOWN => direction.z += 1.0,
                InputCommand::ACTION => todo!(),
            }
        }
        if direction.length() > 0.0 {
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
    mut player_query: Query<&mut AnimationPlayer>,
    mut state_query: Query<&mut PlayerState>,
) {
    let idle_index = 0;
    let run_index = 1;

    if let Ok(mut player) = player_query.get_single_mut() {
        for mut state in state_query.iter_mut() {
            match state.state {
                PlayerStateEnum::IDLE => {
                    if state.animation.is_none() || state.animation.unwrap() != idle_index {
                        player
                            .play(animations.0[idle_index].clone_weak())
                            .set_speed(1.0)
                            .repeat();
                        state.animation = Some(idle_index);
                    }
                }
                PlayerStateEnum::MOVING => {
                    if state.animation.is_none() || state.animation.unwrap() != run_index {
                        player
                            .play(animations.0[run_index].clone_weak())
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
        .add_plugin(InputPlugin)
        .add_startup_system(spawn_system)
        // .add_startup_system_to_stage(StartupStage::PostStartup, debug_spawn_system)
        .add_system(player_movement_system.after(input_system))
        .add_system(player_animation_system.after(player_movement_system))
        .add_system(camera_follow_player_system)
        .run();
}

fn debug_spawn_system(query: Query<(Entity, &Name, &Transform, &MovementSpeed, Option<&Player>)>) {
    for (e, name, transform, speed, player) in query.iter() {
        println!("---------ENTITY----------");
        println!("Entity ID: {:?}", e);
        println!("Name: {}", name.0);
        println!("position: {:?}", transform);
        println!("speed: {}", speed.0);
        if let Some(_player) = player {
            println!("Entity is a Player!!");
        }
        println!("-------------------");
    }
}
