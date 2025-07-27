use std::collections::HashMap;

use bevy::{
    asset::{Asset, AssetLoader, Handle, LoadContext, RenderAssetUsages, io::Reader},
    color::{Color, ColorToComponents},
    image::Image,
    log::error,
    math::{Mat3, Mat4},
    reflect::TypePath,
    render::{
        mesh::{Indices, Mesh, MeshAabb, PrimitiveTopology},
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use copyless::VecHelper;
use flash_runtime::{
    core::AnimationPlayer,
    parse_animation,
    parser::{
        bitmap::CompressedBitmap,
        parse_shape::tessellator::{Draw, DrawType, Gradient},
    },
};
use swf::{CharacterId, GradientInterpolation, Shape};

use crate::{
    ShapeDrawType, ShapeMesh,
    render::material::{BitmapMaterial, GradientMaterial, GradientUniforms, SwfColorMaterial},
};

/// 制作多大得渐变纹理，越大细节越丰富，但是内存占用也越大
const GRADIENT_SIZE: usize = 256;

#[derive(Asset, TypePath)]
pub struct FlashAnimationSwfData {
    pub player: AnimationPlayer,
    pub shape_meshes: HashMap<CharacterId, (Vec<ShapeMesh>, Shape)>,
}

#[derive(Default)]
pub(crate) struct SwfLoader;

impl AssetLoader for SwfLoader {
    type Asset = FlashAnimationSwfData;

    type Settings = ();

    type Error = swf::error::Error;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut swf_data = Vec::new();
        reader.read_to_end(&mut swf_data).await?;

        let (animations, graphics, bitmaps) = parse_animation(swf_data);

        let mut shape_meshes = HashMap::new();

        let mut i = 0;
        let mut j = 0;
        for (k, v) in graphics {
            let lyon_mesh = v.lyon_mesh;

            let gradient_textures =
                load_gradient_textures(lyon_mesh.gradients, load_context, &mut i);
            let draws = lyon_mesh.draws;
            shape_meshes.insert(
                k,
                (
                    load_shape_mesh(draws, gradient_textures, &bitmaps, load_context, &mut j),
                    v.shape,
                ),
            );
        }

        let player = AnimationPlayer::new(
            animations.animations,
            animations.children_clip,
            animations.meta.frame_rate,
        );
        Ok(FlashAnimationSwfData {
            player,
            shape_meshes,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["swf"]
    }
}

fn load_shape_mesh(
    draws: Vec<Draw>,
    gradient_textures: Vec<(Handle<Image>, GradientUniforms)>,
    bitmaps: &HashMap<CharacterId, CompressedBitmap>,
    load_context: &mut LoadContext,
    j: &mut i32,
) -> Vec<ShapeMesh> {
    let mut render_shape = Vec::new();
    for draw in draws {
        match &draw.draw_type {
            DrawType::Color => {
                let mut positions = Vec::with_capacity(draw.vertices.len());
                let mut colors = Vec::with_capacity(draw.vertices.len());
                for vertex in &draw.vertices {
                    positions.alloc().init([vertex.x, vertex.y, 0.0]);
                    let linear_color = Color::srgba_u8(
                        vertex.color.r,
                        vertex.color.g,
                        vertex.color.b,
                        vertex.color.a,
                    )
                    .to_linear();
                    colors.alloc().init(linear_color.to_f32_array());
                }

                let mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
                .with_inserted_indices(Indices::U32(draw.indices.into_iter().collect()));
                let aabb = mesh.compute_aabb().unwrap_or_default();
                let mesh = load_context.add_labeled_asset(format!("mesh_{}", j), mesh);

                render_shape.push(ShapeMesh {
                    mesh,
                    aabb,
                    draw_type: ShapeDrawType::Color(SwfColorMaterial::default()),
                });
            }
            DrawType::Gradient { matrix, gradient } => {
                let mut positions = Vec::with_capacity(draw.vertices.len());
                for vertex in &draw.vertices {
                    positions.alloc().init([vertex.x, vertex.y, 0.0]);
                }
                let mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_indices(Indices::U32(draw.indices.into_iter().collect()));
                let aabb = mesh.compute_aabb().unwrap_or_default();
                let mesh = load_context.add_labeled_asset(format!("mesh_{}", j), mesh);

                let texture = gradient_textures.get(*gradient).unwrap();
                render_shape.push(ShapeMesh {
                    mesh,
                    aabb,
                    draw_type: ShapeDrawType::Gradient(GradientMaterial {
                        gradient: GradientUniforms {
                            focal_point: texture.1.focal_point,
                            interpolation: texture.1.interpolation,
                            shape: texture.1.shape,
                            repeat: texture.1.repeat,
                        },
                        texture_transform: Mat4::from_mat3(Mat3::from_cols_array_2d(&matrix)),
                        texture: Some(texture.0.clone()),
                        ..Default::default()
                    }),
                });
            }
            DrawType::Bitmap(bitmap) => {
                let texture_transform = bitmap.matrix;
                if let Some(compressed_bitmap) = bitmaps.get(&bitmap.bitmap_id) {
                    let decoded = match compressed_bitmap.decode() {
                        Ok(decoded) => decoded,
                        Err(e) => {
                            error!("Failed to decode bitmap: {:?}", e);
                            continue;
                        }
                    };
                    let bitmap = decoded.into_rgba();

                    let bitmap_texture = Image::new(
                        Extent3d {
                            width: bitmap.width(),
                            height: bitmap.height(),
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        bitmap.data().to_vec(),
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::default(),
                    );

                    let mut positions = Vec::with_capacity(draw.vertices.len());
                    for vertex in draw.vertices.clone() {
                        positions.alloc().init([vertex.x, vertex.y, 0.0]);
                    }
                    let mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                    .with_inserted_indices(Indices::U32(draw.indices.into_iter().collect()));
                    let aabb = mesh.compute_aabb().unwrap_or_default();
                    let mesh = load_context.add_labeled_asset(format!("mesh_{}", j), mesh);

                    let handle =
                        load_context.add_labeled_asset(format!("bitmap_{}", j), bitmap_texture);
                    render_shape.push(ShapeMesh {
                        mesh,
                        aabb,
                        draw_type: ShapeDrawType::Bitmap(BitmapMaterial {
                            texture: handle,
                            texture_transform: Mat4::from_mat3(Mat3::from_cols_array_2d(
                                &texture_transform,
                            )),
                            ..Default::default()
                        }),
                    });
                }
            }
        }
        *j += 1;
    }
    render_shape
}

fn load_gradient_textures(
    gradients: Vec<Gradient>,
    load_context: &mut LoadContext,
    i: &mut i32,
) -> Vec<(Handle<Image>, GradientUniforms)> {
    let mut gradient_textures = Vec::new();
    for gradient in gradients {
        let colors = if gradient.records.is_empty() {
            vec![0; GRADIENT_SIZE * 4]
        } else {
            let mut colors = vec![0; GRADIENT_SIZE * 4];
            let convert = if gradient.interpolation == GradientInterpolation::LinearRgb {
                |color| color
            } else {
                |color| color
            };

            for t in 0..GRADIENT_SIZE {
                let mut last = 0;
                let mut next = 0;
                for (i, record) in gradient.records.iter().enumerate().rev() {
                    if (record.ratio as usize) < t {
                        last = i;
                        next = (i + 1).min(gradient.records.len() - 1);
                        break;
                    }
                }
                assert!(last == next || last + 1 == next);
                let last_record = &gradient.records[last];
                let next_record = &gradient.records[next];
                let factor = if next == last {
                    0.0
                } else {
                    (t as f32 - last_record.ratio as f32)
                        / (next_record.ratio as f32 - last_record.ratio as f32)
                };

                colors[t * 4] = lerp(
                    convert(last_record.color.r as f32),
                    convert(next_record.color.r as f32),
                    factor,
                ) as u8;
                colors[(t * 4) + 1] = lerp(
                    convert(last_record.color.g as f32),
                    convert(next_record.color.g as f32),
                    factor,
                ) as u8;
                colors[(t * 4) + 2] = lerp(
                    convert(last_record.color.b as f32),
                    convert(next_record.color.b as f32),
                    factor,
                ) as u8;
                colors[(t * 4) + 3] = lerp(
                    last_record.color.a as f32,
                    next_record.color.a as f32,
                    factor,
                ) as u8;
            }
            colors
        };

        let texture = Image::new(
            Extent3d {
                width: GRADIENT_SIZE as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            colors,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let gradient_uniforms = GradientUniforms::from(gradient);

        let handle = load_context.add_labeled_asset(format!("gradient_{}", i), texture);
        *i += 1;
        gradient_textures.push((handle, gradient_uniforms));
    }
    gradient_textures
}

/// 线性插值
fn lerp(a: f32, b: f32, factor: f32) -> f32 {
    a + (b - a) * factor
}
