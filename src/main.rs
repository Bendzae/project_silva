use animation::player_animation_system;
use bevy::prelude::*;
use camera::camera_follow_player_system;
// use bevy_inspector_egui::prelude::*;
use input::MouseFloorPosition;
use player::{PlayerState, Player, PlayerStateEnum};
use std::f32::consts::PI;
use texture_tiling::TextureTilingPlugin;

use crate::{
    input::{input_system, InputCommand, InputEvent, InputPlugin},
    texture_tiling::{TextureTiling, TileableTextures}, animation::Animations, player::PlayerBundle,
};

mod camera;
mod input;
mod animation;
mod texture_tiling;
mod player;

#[derive(Component)]
pub struct Name(pub String);

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

    let floor_texture_handle = asset_server.load("test_texture.png");

    commands.insert_resource(TileableTextures(vec![floor_texture_handle.clone()]));

    // Floor
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 20.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0).into(),
                base_color_texture: Some(floor_texture_handle.clone()),
                ..default()
            }),
            ..default()
        })
        .insert(TextureTiling { x: 2.0, y: 2.0 });

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

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // .add_plugins_with(DefaultPlugins, |plugins| {
        //     plugins.disable::<AnimationPlugin>()
        // })
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InputPlugin)
        .add_plugin(TextureTilingPlugin)
        .add_startup_system(spawn_system)
        // .add_startup_system_to_stage(StartupStage::PostStartup, debug_spawn_system)
        .add_system(player_movement_system.after(input_system))
        .add_system(player_animation_system.after(player_movement_system))
        .add_system(camera_follow_player_system)
        .run();
}
