use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    color::palettes::tailwind::*,
    image::ImageSampler,
    math::sampling::UniformMeshSampler,
    platform::collections::HashMap,
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        render_resource::{
            Extent3d, TextureDimension, TextureFormat,
        },
    },
};
use noiz::prelude::*;
use rand::{prelude::Distribution, thread_rng};

use crate::{AppState, playing::Player};

const CHUNK_SIZE: f32 = 200.;
pub const TERRAIN_AMPLITUDE: f32 = 20.;

pub struct LandChunkPlugin;

impl Plugin for LandChunkPlugin {
    fn build(&self, app: &mut App) {
        let mut perlin_noise = Noise::<(
            // mixes gradients from `QuickGradients` (a
            // lookup table) across each cell
            // via a smoothstep, where each
            // cell is on an orthogonal (cartesian) grid,
            MixCellGradients<
                OrthoGrid,
                Smoothstep,
                QuickGradients,
            >,
            // then maps those snorm values to unorm.
            SNormToUNorm,
        )>::default();
        perlin_noise.set_seed(12345); // Any seed will do. Even 0 is fine!

        app.insert_resource(LandChunkNoise(perlin_noise))
            .init_resource::<LoadedChunks>()
            .add_systems(Startup, gen_debug_material)
            .add_systems(
                Update,
                ensure_land_chunks
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource)]
struct DebugMaterial(Handle<StandardMaterial>);

fn gen_debug_material(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // noise: Res<LandChunkNoise>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(
            images.add(uv_debug_texture()),
        ),
        ..default()
    });
    commands.insert_resource(DebugMaterial(debug_material));
}

#[derive(Component)]
pub struct LandChunk;

#[derive(Resource, Deref, DerefMut)]
pub struct LandChunkNoise(
    Noise<(
        // mixes gradients from `QuickGradients` (a lookup
        // table) across each cell via a
        // smoothstep, where each cell is on an
        // orthogonal (cartesian) grid,
        MixCellGradients<
            OrthoGrid,
            Smoothstep,
            QuickGradients,
        >,
        // then maps those snorm values to unorm.
        SNormToUNorm,
    )>,
);

#[derive(Resource, Default)]
struct LoadedChunks(HashMap<u32, Entity>);

#[derive(Component)]
pub struct Obstacle;

fn ensure_land_chunks(
    query: Single<&Transform, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    noise: Res<LandChunkNoise>,
    debug_material: Res<DebugMaterial>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let offset =
        (query.translation.z / CHUNK_SIZE).abs() as u32;
    // info!(?offset);
    for offset in offset..(offset + 5) {
        if loaded_chunks.0.get(&offset).is_none() {
            let chunk = gen_land_chunk(
                CHUNK_SIZE * offset as f32,
                &noise,
            );

            let triangles = chunk.triangles().unwrap();
            let sampler =
                UniformMeshSampler::try_new(triangles)
                    .unwrap();
            let rng = thread_rng();
            // let rng = StdRng::;
            let samples: Vec<Vec3> =
                sampler.sample_iter(rng).take(2).collect();

            let id = commands
                .spawn((
                    Name::new("LandChunk"),
                    LandChunk,
                    Mesh3d(meshes.add(chunk)),
                    MeshMaterial3d(
                        debug_material.0.clone(),
                    ),
                    Transform::from_xyz(
                        0.,
                        0.0,
                        -CHUNK_SIZE * offset as f32,
                    ),
                    RigidBody::Static,
                    ColliderConstructor::TrimeshFromMesh,
                    CollisionMargin(0.1),
                    // collider,
                ))
                .id();

            loaded_chunks.0.insert(offset, id);

            for sample in samples {
                info!(
                    ?sample,
                    z = sample.z
                        + CHUNK_SIZE * offset as f32,
                );
                commands.spawn((
                    Name::new("Obstacle"),
                    Obstacle,
                    Collider::cuboid(30., 30., 30.),
                    RigidBody::Static,
                    Mesh3d(
                        meshes.add(Cuboid::new(
                            30., 30., 30.,
                        )),
                    ),
                    MeshMaterial3d(materials.add(
                        StandardMaterial {
                            base_color: RED_400.into(),
                            ..default()
                        },
                    )),
                    Transform::from_xyz(
                        sample.x,
                        sample.y,
                        sample.z
                            - CHUNK_SIZE * offset as f32,
                    ),
                ));
            }
        }
    }
}

fn gen_land_chunk(
    offset: f32,
    noise: &LandChunkNoise,
) -> Mesh {
    let subdivisions = 64;

    let mut plane = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(CHUNK_SIZE, CHUNK_SIZE)
            .subdivisions(subdivisions),
    );

    let pos_attribute = plane
        .attribute_mut(Mesh::ATTRIBUTE_POSITION)
        .unwrap();
    let VertexAttributeValues::Float32x3(pos_attribute) =
        pos_attribute
    else {
        panic!(
            "Unexpected vertex format, expected Float32x2."
        );
    };

    pos_attribute.iter_mut().for_each(|arr| {
        let some_value: f32 = noise.sample(Vec3::new(
            arr[0] / 80.,
            arr[1] / 10.,
            (arr[2] - offset) / 80.,
        ));
        // let some_value =
        //     ((arr[2] - offset) / 10.).sin() * 1.;
        arr[1] = some_value * TERRAIN_AMPLITUDE;
    });

    // TODO: Maybe switch to heightfield
    // info!(count = pos_attribute.len());
    // let heights: Vec<Vec<f32>> = pos_attribute
    //     .iter()
    //     .map(|arr| arr[1])
    //     .chunks((subdivisions + 2) as usize)
    //     .into_iter()
    //     .map(|chunk| chunk.collect())
    //     .collect();

    // (
    plane.compute_smooth_normals();
    plane
    //     Collider::heightfield(heights,
    // Vec3::ONE), )
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255,
        102, 255, 121, 255, 102, 255, 102, 255, 198, 255,
        102, 198, 255, 255, 121, 102, 255, 255, 236, 102,
        255, 255,
    ];

    let mut texture_data =
        [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)]
            .copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}
