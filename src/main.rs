use animation::player_animation_system;
use bevy::prelude::*;

use bevy::render::texture::ImageSettings;
use bevy_inspector_egui::prelude::*;
use camera::camera_follow_player_system;
use input::input_system;
use movement::{player_movement_system, MovementSpeed};
use test_scene::TestScencePlugin;
use texture_tiling::TextureTilingPlugin;

use crate::input::InputPlugin;

mod animation;
mod camera;
mod input;
mod movement;
mod player;
mod test_scene;
mod texture_tiling;
mod enemy;

mod debug;

#[derive(Component, Inspectable)]
pub struct NameV2(pub String);

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ImageSettings::default_linear())
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(TestScencePlugin)
        .add_plugin(InputPlugin)
        .add_plugin(TextureTilingPlugin)
        .add_system(player_movement_system.after(input_system))
        .add_system(player_animation_system.after(player_movement_system))
        .add_system(camera_follow_player_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}
