use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::utils::tracing::Event;

use crate::input::{InputCommand, InputEvent, InputPlugin};

mod input;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct MovementSpeed(f32);

#[derive(Bundle)]
struct PlayerBundle {
    _p: Player,
    name: Name,
    movement_speed: MovementSpeed,
    #[bundle]
    pbr_bundle: PbrBundle,
}

impl Default for PlayerBundle {
    fn default() -> PlayerBundle {
        return PlayerBundle {
            _p: Player,
            name: Name("unknown".to_string()),
            movement_speed: MovementSpeed(2.0),
            pbr_bundle: PbrBundle::default(),
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

fn spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Floor
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
        ..default()
    });

    let player_pbr_bundle = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::RED,
            perceptual_roughness: 1.0,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    };

    commands.spawn_bundle(PlayerBundle {
        name: Name("Player_1".to_string()),
        pbr_bundle: player_pbr_bundle,
        ..PlayerBundle::default()
    });

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
        transform: Transform::from_xyz(0.0, 8.0, 0.0)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, -(PI / 4.0), (PI / 8.0), 0.0)),
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
    mut query: Query<(&mut Transform, &MovementSpeed), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, speed) in query.iter_mut() {
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
            transform.translation += direction.normalize() * speed.0 * time.delta_seconds();
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(InputPlugin)
        .add_startup_system(spawn_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, debug_spawn_system)
        .add_system(player_movement_system)
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
