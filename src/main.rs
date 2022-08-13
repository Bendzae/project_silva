use animation::player_animation_system;
use bevy::prelude::*;
use bevy::render::texture::ImageSettings;
use bevy::scene::SceneInstance;
use bevy::window::WindowSettings;
use bevy_inspector_egui::prelude::*;
use camera::{camera_follow_player_system};
use movement::{player_movement_system, MovementSpeed};
use player::Player;
use std::f32::consts::PI;
use texture_tiling::TextureTilingPlugin;

use crate::input::InputPlugin;
use crate::{
    animation::Animations,
    debug::TestBundle,
    input::input_system,
    player::PlayerBundle,
    texture_tiling::{TextureTiling, TileableTextures},
};

mod animation;
mod camera;
mod input;
mod movement;
mod player;
mod texture_tiling;

mod debug;

#[derive(Component)]
pub struct NameV2(pub String);

#[derive(Component)]
struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    _e: Enemy,
    name: NameV2,
    transform: Transform,
    movement_speed: MovementSpeed,
}

impl Default for EnemyBundle {
    fn default() -> EnemyBundle {
        return EnemyBundle {
            _e: Enemy,
            name: NameV2("unknown".to_string()),
            transform: Transform::default(),
            movement_speed: MovementSpeed(1.0),
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

    let floor_texture_handle = asset_server.load("test_textures/Dark/texture_06.png");
    // let ao_test = asset_server.load("ao_test.png");
    let normal_map_test = asset_server.load("normal_test.png");

    commands.insert_resource(TileableTextures(vec![
        floor_texture_handle.clone(),
        normal_map_test.clone(),
    ]));

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

    (0..2).for_each(|x| {
        (0..2).for_each(|z| {
            commands
                .spawn_bundle(MaterialMeshBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
                    transform: Transform::from_xyz(
                        (x * 10 - 5) as f32,
                        1.0 - ((x + z) as f32 / 2.0),
                        (z * 10 - 5) as f32,
                    ),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(1.0, 1.0, 1.0).into(),
                        // base_color_texture: Some(normal_map_test.clone()),
                        // occlusion_texture: Some(ao_test.clone()),
                        normal_map_texture: Some(normal_map_test.clone()),
                        perceptual_roughness: 1.0,
                        metallic: 0.1,
                        // base_color_texture: Some(floor_texture_handle.clone()),
                        ..default()
                    }),
                    ..default()
                })
                .insert(TextureTiling { x: 1.0, y: 1.0 });
        });
    });

    // Box
    commands.spawn_bundle(TestBundle {
        pbr_bundle: PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
            material: materials.add(Color::rgb(0.8, 0.5, 0.5).into()),
            ..default()
        },
        ..Default::default()
    });

    // Player
    commands
        .spawn_bundle(PlayerBundle {
            name: NameV2("Player_1".to_string()),
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
        })
        .with_children(|parent| {
            parent.spawn_bundle(PointLightBundle {
                transform: Transform::from_xyz(0., 0.6, 0.),
                point_light: PointLight {
                    color: Color::rgb(0., 0.8, 1.0),
                    intensity: 200.0,
                    ..default()
                },
                ..default()
            });
        });

    // Enemies
    for i in 0..3 {
        commands.spawn_bundle(EnemyBundle {
            name: NameV2(format!("enemy_{i}")),
            ..EnemyBundle::default()
        });
    }

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.4,
    });

    // point light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(-10., 3., 0.),
        ..default()
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
        transform: Transform::from_xyz(-6.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn set_material_system(
    query: Query<(Entity, &Handle<Mesh>, &Name)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut ran: Local<bool>,
) {
    if *ran {
        return;
    }
    for (e, handle, name) in query.iter() {
        println!("{:?}", name);
        if let Some(mesh) = meshes.get_mut(handle) {
            let material_brows = materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.0, 0.0).into(),
                ..default()
            });
            let material_body = materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0).into(),
                // perceptual_roughness: 1.0,
                ..default()
            });
            let material_eyes = materials.add(StandardMaterial {
                base_color: Color::rgb(0.1, 0.8, 1.0).into(),
                emissive: Color::rgb(0.0, 0.8, 1.0).into(),
                ..default()
            });

            commands.entity(e).remove::<Handle<StandardMaterial>>();

            match mesh.indices().unwrap().len() {
                120 => commands.entity(e).insert(material_brows),
                2436 => commands.entity(e).insert(material_body),
                168 => commands.entity(e).insert(material_eyes),
                _ => commands.entity(e).insert(material_brows),
            };
        }
        *ran = true;
    }
}

fn prepate_meshes(handles: Query<&Handle<Mesh>>, mut meshes: ResMut<Assets<Mesh>>) {
    for handle in handles.iter() {
        if let Some(mesh) = meshes.get_mut(handle) {
            mesh.generate_tangents().unwrap();
        }
    }
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ImageSettings::default_linear())
        .add_plugins(DefaultPlugins)
        // .add_plugins_with(DefaultPlugins, |plugins| {
        //     plugins.disable::<AnimationPlugin>()
        // })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InputPlugin)
        .add_plugin(TextureTilingPlugin)
        .add_startup_system(spawn_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, prepate_meshes)
        .add_system(player_movement_system.after(input_system))
        .add_system(player_animation_system.after(player_movement_system))
        .add_system(camera_follow_player_system)
        .add_system(set_material_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}
