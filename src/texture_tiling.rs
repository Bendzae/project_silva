use bevy::{
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
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
    mut ev_asset: EventReader<AssetEvent<Mesh>>,
    mut query: Query<(&TextureTiling, &mut Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                let mesh = meshes
                    .get_mut(&handle)
                    .expect("falied to get loaded mesh asset");

                mesh.generate_tangents()
                    .expect("Failed to generate tangents");

                for (tiling_info, mesh_handle) in query.iter_mut() {
                    if *mesh_handle == *handle {
                        let mesh = meshes
                            .get_mut(&mesh_handle)
                            .expect("falied to get loaded mesh asset");

                        if let Some(VertexAttributeValues::Float32x2(uvs)) =
                            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
                        {
                            for uv in uvs {
                                uv[0] *= tiling_info.x;
                                uv[1] *= tiling_info.y;
                            }

                            println!("modified uv's for mesh")
                        }
                    }
                }
            }
            AssetEvent::Modified { handle } => (),
            AssetEvent::Removed { handle } => (),
        }
    }
}

fn image_config_system(
    mut ev_asset: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    textures: Res<TileableTextures>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                // a texture was just loaded or changed!

                // WARNING: this mutable access will cause another
                // AssetEvent (Modified) to be emitted!
                let image = images.get_mut(handle).unwrap();
                // ^ unwrap is OK, because we know it is loaded now

                if textures.0.contains(&*handle) {
                    // It's a tileable texture
                    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        mipmap_filter: FilterMode::Linear,
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        ..Default::default()
                    });
                    println!("Ran image_config_system");
                    for (_, mut mat) in materials.iter_mut() {
                        // Weird hack needed to force material to update TODO: Find better way
                        let col = mat.base_color;
                        mat.base_color = col; 
                    }
                }
            }
            AssetEvent::Modified { handle } => {
                // an image was modified
            }
            AssetEvent::Removed { handle } => {
                // an image was unloaded
            }
        }
    }
}

pub struct TextureTilingPlugin;

impl Plugin for TextureTilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(tiling_system)
            .add_system(image_config_system);
    }

    fn name(&self) -> &str {
        "TextureTilingPlugin"
    }
}
