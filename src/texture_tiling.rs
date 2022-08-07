use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{AddressMode, SamplerDescriptor, FilterMode},
        texture::ImageSampler,
    },
};
// All textures that should be set to address mode REPEAT need to go in this Resource
pub struct TileableTextures(pub Vec<Handle<Image>>);

#[derive(Component)]
pub struct TextureTiling {
    pub x: f32,
    pub y: f32,
}

fn tiling_system(
    mut query: Query<(&TextureTiling, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (tiling_info, mut mesh_handle) in query.iter_mut() {
        if let Some(mesh) = meshes.get_mut(&mesh_handle) {
            if let Some(VertexAttributeValues::Float32x2(uvs)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                for uv in uvs {
                    uv[0] *= tiling_info.x;
                    uv[1] *= tiling_info.y;
                }
            }
        }
    }
}

fn image_config_system(
    textures: Res<TileableTextures>,
    mut images: ResMut<Assets<Image>>,
    mut ran: Local<bool>,
) {
    if *ran {
        return;
    }
    for texture in &textures.0 {
        let image = images.get_mut(&texture.clone());
        if let Some(image) = image {
            image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                ..Default::default()
            });
            *ran = true;
        }
    }
}

pub struct TextureTilingPlugin;

impl Plugin for TextureTilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, tiling_system)
            .add_system(image_config_system);
    }

    fn name(&self) -> &str {
        "TextureTilingPlugin"
    }
}
