use bevy::asset::{RenderAssetUsages, weak_handle};
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommands, Local};
use bevy::image::{BevyDefault, Image};
use bevy::math::{UVec2, Vec2, Vec4};
use bevy::platform::collections::HashMap;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{
    CachedRenderPipelineId, Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::NoFrustumCulling;
use bevy::transform::components::GlobalTransform;
use bevy::{
    app::{App, Plugin, Update},
    asset::{Assets, Handle, load_internal_asset},
    math::{Mat4, Vec3},
    prelude::{
        Children, Commands, Component, Deref, DerefMut, Entity, Mesh, Mesh2d, Query, ResMut,
        Shader, Transform, Visibility, With, Without,
    },
    render::RenderApp,
    sprite::{Material2dPlugin, MeshMaterial2d},
};
use blend_pipeline::BlendType;
use flash_runtime::core::filter::Filter;
use graph::FlashFilterRenderGraphPlugin;
use intermediate_texture::{IntermediateTexture, IntermediateTexturePlugin, SwfRawVertex};
use material::{
    BitmapMaterial, BlendMaterialKey, GradientMaterial, GradientUniforms, SwfColorMaterial,
    SwfMaterial, SwfTransform,
};
use pipeline::IntermediateTexturePipeline;
use swf::{CharacterId, Rectangle as SwfRectangle, Twips};

use crate::assets::FlashAnimationSwfData;
use crate::bundle::{FlashAnimation, FlashShapeSpawnRecord, SwfGraph};
use crate::{FlashAnimationActiveInstance, ShapeDrawType, flash_update};

pub(crate) mod blend_pipeline;
mod graph;
mod intermediate_texture;
pub(crate) mod material;
mod pipeline;

pub const SWF_COLOR_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("8c2a5b0f-3e6d-4f8a-b217-84d2f5e1c9b3");
pub const GRADIENT_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("5e9f1a78-9b34-4c15-8d7e-2a3b0f47d862");
pub const BITMAP_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("a34c7d82-1f5b-4a9e-93d8-6b7e20c45a1f");
pub const FLASH_COMMON_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("e53b9f82-6a4c-4d5b-91e7-4f2a63b8c5d9");

type SwfShapeMeshQuery = (
    Entity,
    Option<&'static MeshMaterial2d<SwfColorMaterial>>,
    Option<&'static MeshMaterial2d<GradientMaterial>>,
    Option<&'static MeshMaterial2d<BitmapMaterial>>,
);

pub struct FlashRenderPlugin;

impl Plugin for FlashRenderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            FLASH_COMMON_MATERIAL_SHADER_HANDLE,
            "render/shaders/common.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SWF_COLOR_MATERIAL_SHADER_HANDLE,
            "render/shaders/color.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GRADIENT_MATERIAL_SHADER_HANDLE,
            "render/shaders/gradient.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            BITMAP_MATERIAL_SHADER_HANDLE,
            "render/shaders/bitmap.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(Material2dPlugin::<GradientMaterial>::default())
            .add_plugins(Material2dPlugin::<SwfColorMaterial>::default())
            .add_plugins(Material2dPlugin::<BitmapMaterial>::default())
            .add_plugins((IntermediateTexturePlugin, FlashFilterRenderGraphPlugin))
            .add_plugins(ExtractComponentPlugin::<FlashFilters>::default())
            .add_systems(Update, generate_or_update_mesh.after(flash_update));
    }

    fn finish(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<IntermediateTexturePipeline>();
        }
    }
}

#[derive(Component, Default)]
pub struct SwfShapeChildMesh;

/// flash 动画滤镜数据
#[derive(Component, Clone, Debug, Default, ExtractComponent, DerefMut, Deref)]
pub struct FlashFilters(Vec<Filter>);

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ShapeFilterInfo {
    pub matrix: swf::Matrix,
    pub id: CharacterId,
    pub shape_ref: usize,
    pub filter_size: UVec2,
}

/// 每帧生成或更新`flash`动画的网格实体
#[allow(clippy::too_many_arguments)]
fn generate_or_update_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut flashes: ResMut<Assets<FlashAnimationSwfData>>,
    mut swf_color_materials: ResMut<Assets<SwfColorMaterial>>,
    mut gradient_materials: ResMut<Assets<GradientMaterial>>,
    mut bitmap_materials: ResMut<Assets<BitmapMaterial>>,
    mut query: Query<(
        Entity,
        &FlashAnimation,
        &mut FlashAnimationActiveInstance,
        &GlobalTransform,
        Option<&Children>,
    )>,
    mut flash_material_query: Query<SwfShapeMeshQuery, With<SwfShapeChildMesh>>,
    mut intermediate_textures: Query<
        (
            Entity,
            &mut IntermediateTexture,
            &mut FlashFilters,
            &MeshMaterial2d<BitmapMaterial>,
        ),
        Without<SwfShapeChildMesh>,
    >,
    mut shape_query: Query<
        (Entity, Option<&Children>, &mut Transform),
        (With<SwfGraph>, Without<SwfShapeChildMesh>),
    >,
    mut current_shape_entities: Local<Vec<Entity>>,
    mut flash_shape_records: Local<EntityHashMap<FlashShapeSpawnRecord>>,
    mut cache_filter_image: Local<EntityHashMap<HashMap<ShapeFilterInfo, Handle<Image>>>>,
    mut live_flash: Local<Vec<Entity>>,
) {
    live_flash.clear();
    for (flash_entity, flash_animation, mut active_instances, global_transform, children) in
        query.iter_mut()
    {
        // 标记当前帧的flash还在使用
        live_flash.push(flash_entity);

        let flash_shape_record = flash_shape_records.entry(flash_entity).or_default();

        // 记录当前帧的flash动画
        current_shape_entities.clear();
        // 防止Shape被多次使用时，引用到同一个实体
        let mut marker_shape_ref = HashMap::new();
        // 将当前flash动画下的中间纹理标记为不可见
        if let Some(children) = children {
            intermediate_textures
                .iter_mut()
                .filter(|(entity, _, _, _)| children.contains(entity))
                .for_each(|(_, mut intermediate_texture, _, _)| {
                    intermediate_texture.is_active = false;
                });
        }
        if let Some(flash) = flashes.get_mut(flash_animation.swf.id()) {
            let mut current_entity = Vec::new();
            let mut z_index = 0.0;
            for active_instance in active_instances.iter_mut() {
                // 记录同一个实体的引用计数
                let ref_count = marker_shape_ref.entry(active_instance.id()).or_default();
                *ref_count += 1;

                // 记录当前生成的实体
                current_entity.push(active_instance.id().clone());
                // 指定的shape id
                let id = active_instance.id();
                // flash 动画变换数据
                let swf_transform = SwfTransform {
                    world_transform: active_instance.transform(),
                    mult_color: Vec4::from_array(
                        active_instance.color_transform().mult_rgba_normalized(),
                    ),
                    add_color: Vec4::from_array(
                        active_instance.color_transform().add_rgba_normalized(),
                    ),
                };
                // flash 混合模式
                let blend_key: BlendMaterialKey = BlendType::from(active_instance.blend()).into();
                // 获取当前实例的swf变换矩阵，用于计算filter_rect
                let matrix = active_instance.transform_matrix();
                // 提取当前实例的滤镜
                active_instance.filters_mut().retain(|f| !f.impotent());
                let filters = active_instance.filters_mut();

                let current_shape_entity;
                if let Some(entity) = flash_shape_record.get_entity(id, *ref_count) {
                    // shape实体已经生成。只需要更新其Mesh2d
                    let (shape_entity, shape_children, mut transform) = shape_query
                        .iter_mut()
                        .find(|(shape_entity, _, _)| shape_entity == entity)
                        .expect("找不到有鬼");
                    current_shape_entity = shape_entity;
                    transform.translation.z = z_index;
                    z_index += 1e-3;
                    // 更新中间纹理变换
                    if let Some((_, mut intermediate_texture, mut flash_filters, material_handle)) =
                        intermediate_textures
                            .iter_mut()
                            .find(|(entity, _, _, _)| *entity == shape_entity)
                    {
                        let Some(bitmap_material) = bitmap_materials.get_mut(material_handle.id())
                        else {
                            continue;
                        };

                        let (_, shape) = flash.shape_meshes.get(&id).expect("没有就是有Bug");
                        let scale = global_transform.scale();
                        let bounds = matrix * shape.shape_bounds.clone();
                        let size = get_graphic_raw_size(&bounds, scale);
                        let filter_rect = get_filter_rect(&bounds, filters, scale);
                        let width = filter_rect.width() as f32;
                        let height = filter_rect.height() as f32;
                        let filter_size =
                            UVec2::new((width * scale.x) as u32, (height * scale.y) as u32);
                        let tx = matrix.tx.to_pixels() as f32;
                        let ty = matrix.ty.to_pixels() as f32;
                        let offset_x = bounds.x_min.to_pixels() as f32 - tx;
                        let offset_y = bounds.y_min.to_pixels() as f32 - ty;
                        let world_transform = Mat4::from_cols_array_2d(&[
                            [matrix.a, matrix.b, 0.0, 0.0],
                            [matrix.c, matrix.d, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [-offset_x, -offset_y, 0.0, 1.0],
                        ]);
                        let image = get_target_image(&filter_size);
                        let image_handle = images.add(image);
                        if let Some(shape_filters) = cache_filter_image.get_mut(&flash_entity) {
                            // 如果当前实例的滤镜和上次的滤镜一样，不需要重新渲染
                            if let Some(cache_image) = shape_filters.get(&ShapeFilterInfo {
                                matrix: matrix.into(),
                                id,
                                shape_ref: *ref_count,
                                filter_size,
                            }) {
                                intermediate_texture.is_active = false;
                                bitmap_material.texture = cache_image.clone();
                            } else {
                                flash_filters.clear();
                                flash_filters.append(filters);
                                intermediate_texture.is_active = true;
                                intermediate_texture.target = image_handle.clone().into();
                                intermediate_texture.filter_size = filter_size;
                                intermediate_texture.world_transform = world_transform;
                                intermediate_texture.size = size;
                                intermediate_texture.scale = global_transform.scale();
                                bitmap_material.texture = image_handle.clone();

                                // 记录新的滤镜
                                shape_filters.insert(
                                    ShapeFilterInfo {
                                        matrix: matrix.into(),
                                        id: active_instance.id(),
                                        shape_ref: *ref_count,
                                        filter_size,
                                    },
                                    image_handle,
                                );
                            }
                        }

                        transform.translation.z += scale.y;

                        let draw_offset =
                            Vec2::new(filter_rect.x_min as f32, filter_rect.y_min as f32);
                        let world_transform = Mat4::from_cols_array_2d(&[
                            [width, 0.0, 0.0, 0.0],
                            [0.0, height, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [
                                tx + offset_x + draw_offset.x,
                                ty + offset_y + draw_offset.y,
                                0.0,
                                1.0,
                            ],
                        ]);

                        let swf_transform = SwfTransform {
                            world_transform,
                            ..swf_transform
                        };

                        bitmap_material.update_swf_material(swf_transform);
                    } else {
                        let Some(shape_children) = shape_children else {
                            continue;
                        };
                        shape_children.iter().for_each(|child| {
                            for (
                                material_entity,
                                swf_color_material_handle,
                                swf_gradient_material_handle,
                                swf_bitmap_material_handle,
                            ) in flash_material_query.iter_mut()
                            {
                                if material_entity == *child {
                                    if let Some(handle) = swf_color_material_handle {
                                        update_swf_material(
                                            handle,
                                            &mut swf_color_materials,
                                            swf_transform.clone(),
                                            blend_key,
                                        );
                                        break;
                                    }
                                    if let Some(handle) = swf_gradient_material_handle {
                                        update_swf_material(
                                            handle,
                                            &mut gradient_materials,
                                            swf_transform.clone(),
                                            blend_key,
                                        );
                                        break;
                                    }
                                    if let Some(handle) = swf_bitmap_material_handle {
                                        update_swf_material(
                                            handle,
                                            &mut bitmap_materials,
                                            swf_transform.clone(),
                                            blend_key,
                                        );
                                        break;
                                    }
                                }
                            }
                        });
                    }
                } else {
                    // 不存在缓存实体
                    let mut shape_entity_command = commands.spawn(SwfGraph);
                    let shape_entity = shape_entity_command.id();

                    let (shape_meshes, shape) = flash.shape_meshes.get(&id).expect("没有就是有Bug");
                    // 是否含有滤镜效果
                    if !filters.is_empty() {
                        // 用于渲染出中间纹理的数据
                        let mut view_entities = Vec::new();
                        shape_meshes.iter().for_each(|shape| {
                            view_entities.push(SwfRawVertex {
                                mesh: shape.mesh.clone(),
                                pipeline_id: CachedRenderPipelineId::INVALID,
                                mesh_draw_type: shape_to_intermediate_texture_draw_type(
                                    &shape.draw_type,
                                ),
                            });
                        });
                        let scale = global_transform.scale();
                        let bounds = matrix * shape.shape_bounds.clone();
                        let size = get_graphic_raw_size(&bounds, scale);
                        let filter_rect = get_filter_rect(&bounds, filters, scale);
                        let width = filter_rect.width() as f32;
                        let height = filter_rect.height() as f32;
                        let filter_size =
                            UVec2::new((width * scale.x) as u32, (height * scale.y) as u32);

                        let tx = matrix.tx.to_pixels() as f32;
                        let ty = matrix.ty.to_pixels() as f32;
                        let offset_x = bounds.x_min.to_pixels() as f32 - tx;
                        let offset_y = bounds.y_min.to_pixels() as f32 - ty;
                        let world_transform = Mat4::from_cols_array_2d(&[
                            [matrix.a, matrix.b, 0.0, 0.0],
                            [matrix.c, matrix.d, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [-offset_x, -offset_y, 0.0, 1.0],
                        ]);
                        let image = get_target_image(&filter_size);
                        let image_handle = images.add(image);
                        shape_entity_command.insert((
                            IntermediateTexture {
                                target: image_handle.clone().into(),
                                is_active: true,
                                scale,
                                size,
                                filter_size,
                                view_entities,
                                world_transform,
                            },
                            FlashFilters(filters.clone()),
                        ));

                        // 初始化当前动画的滤镜缓存
                        cache_filter_image.entry(flash_entity).or_default();

                        let draw_offset =
                            Vec2::new(filter_rect.x_min as f32, filter_rect.y_min as f32);
                        let world_transform = Mat4::from_cols_array_2d(&[
                            [width, 0.0, 0.0, 0.0],
                            [0.0, height, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [
                                tx + offset_x + draw_offset.x,
                                ty + offset_y + draw_offset.y,
                                0.0,
                                1.0,
                            ],
                        ]);
                        let swf_transform = SwfTransform {
                            world_transform,
                            ..swf_transform
                        };
                        let mesh = generate_rectangle_mesh_and_texture_transform();
                        shape_entity_command.insert((
                            Mesh2d(meshes.add(mesh)),
                            MeshMaterial2d(bitmap_materials.add(BitmapMaterial {
                                blend_key,
                                texture: image_handle.clone(),
                                texture_transform: Mat4::IDENTITY,
                                transform: swf_transform.clone(),
                            })),
                            // Transform::from_translation(Vec3::new(0., 0., scale.y + z_index)),
                        ));
                    } else {
                        // 生成网格实体
                        shape_meshes.iter().for_each(|shape_mesh| {
                            // 防止Shape中的绘制z冲突
                            z_index += 1e-3;
                            let transform =
                                Transform::from_translation(Vec3::new(0.0, 0.0, z_index));
                            match &shape_mesh.draw_type {
                                ShapeDrawType::Color(swf_color_material) => {
                                    spawn_mesh(
                                        &mut shape_entity_command,
                                        swf_color_material.clone(),
                                        &mut swf_color_materials,
                                        swf_transform.clone(),
                                        transform,
                                        shape_mesh.mesh.clone(),
                                        blend_key,
                                    );
                                }
                                ShapeDrawType::Gradient(gradient_material) => {
                                    spawn_mesh(
                                        &mut shape_entity_command,
                                        gradient_material.clone(),
                                        &mut gradient_materials,
                                        swf_transform.clone(),
                                        transform,
                                        shape_mesh.mesh.clone(),
                                        blend_key,
                                    );
                                }
                                ShapeDrawType::Bitmap(bitmap_material) => {
                                    spawn_mesh(
                                        &mut shape_entity_command,
                                        bitmap_material.clone(),
                                        &mut bitmap_materials,
                                        swf_transform.clone(),
                                        transform,
                                        shape_mesh.mesh.clone(),
                                        blend_key,
                                    );
                                }
                            }
                        });
                    }

                    current_shape_entity = shape_entity;
                    flash_shape_record.mark_cached_shape(
                        active_instance.id(),
                        *ref_count,
                        shape_entity,
                    );
                    commands.entity(flash_entity).add_child(shape_entity);
                }
                current_shape_entities.push(current_shape_entity);
            }

            // 每帧隐藏所有实体
            flash_shape_record
                .cache_entities()
                .iter()
                .for_each(|(_, entity)| {
                    commands.entity(*entity).insert(Visibility::Hidden);
                });
            // 将当前帧所含有的实体设置为可见
            current_shape_entities.iter().for_each(|entity| {
                commands.entity(*entity).insert(Visibility::Inherited);
            });
        }
    }
    // 仅保留还活跃的flash动画实体
    cache_filter_image.retain(|entity, _| live_flash.contains(entity));
    flash_shape_records.retain(|entity, _| live_flash.contains(entity));
}

#[derive(Clone, Default)]
pub struct Gradient {
    pub gradient: GradientUniforms,
    pub texture: Handle<Image>,
    pub texture_transform: Mat4,
}

#[derive(Clone)]
pub enum RawVertexDrawType {
    Color,
    Gradient(Gradient),
    Bitmap,
}
impl Default for RawVertexDrawType {
    fn default() -> Self {
        RawVertexDrawType::Color
    }
}

fn shape_to_intermediate_texture_draw_type(draw_type: &ShapeDrawType) -> RawVertexDrawType {
    let draw_type = draw_type.clone();
    match draw_type {
        ShapeDrawType::Color(_) => RawVertexDrawType::Color,
        ShapeDrawType::Gradient(gradient_material) => RawVertexDrawType::Gradient(Gradient {
            texture: gradient_material
                .texture
                .clone()
                .expect("渐变纹理应该已经生成！！！"),
            gradient: gradient_material.gradient,
            texture_transform: gradient_material.texture_transform,
        }),
        ShapeDrawType::Bitmap(_) => RawVertexDrawType::Bitmap,
    }
}

#[inline]
fn update_swf_material<T: SwfMaterial>(
    handle: &Handle<T>,
    swf_materials: &mut ResMut<Assets<T>>,
    swf_transform: SwfTransform,
    blend_key: BlendMaterialKey,
) {
    // 当缓存某实体后该实体在该系统尚未运行完成时会查询不到对应的材质，此时重新生成材质。
    if let Some(swf_material) = swf_materials.get_mut(handle) {
        swf_material.update_swf_material(swf_transform);
        swf_material.set_blend_key(blend_key);
    }
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn spawn_mesh<T: SwfMaterial>(
    commands: &mut EntityCommands,
    mut swf_material: T,
    swf_materials: &mut ResMut<Assets<T>>,
    swf_transform: SwfTransform,
    transform: Transform,
    handle: Handle<Mesh>,
    blend_key: BlendMaterialKey,
) {
    swf_material.update_swf_material(swf_transform);
    swf_material.set_blend_key(blend_key);
    commands.with_children(|parent| {
        parent.spawn((
            Mesh2d(handle),
            MeshMaterial2d(swf_materials.add(swf_material)),
            transform,
            SwfShapeChildMesh,
            // 由于Flash顶点特殊性不应用剔除
            NoFrustumCulling,
        ));
    });
}

fn get_graphic_raw_size(bounds: &SwfRectangle<Twips>, global_scale: Vec3) -> UVec2 {
    let width = bounds.width().to_pixels().ceil().max(0.0) as f32;
    let height = bounds.height().to_pixels().ceil().max(0.0) as f32;
    let width = (width * global_scale.x) as u32;
    let height = (height * global_scale.y) as u32;
    UVec2::new(width, height)
}

fn get_filter_rect(
    bounds: &SwfRectangle<Twips>,
    filters: &mut Vec<Filter>,
    global_scale: Vec3,
) -> SwfRectangle<i32> {
    let scale_x = global_scale.x;
    let scale_y = global_scale.y;

    let width = bounds.width().to_pixels().ceil().max(0.0);
    let height = bounds.height().to_pixels().ceil().max(0.0);
    let mut filter_rect = SwfRectangle {
        x_min: Twips::ZERO,
        x_max: Twips::from_pixels_i32(width as i32),
        y_min: Twips::ZERO,
        y_max: Twips::from_pixels_i32(height as i32),
    };
    for filter in filters {
        filter.scale(scale_x, scale_y);
        filter_rect = filter.calculate_dest_rect(filter_rect);
    }

    let filter_rect = SwfRectangle {
        x_min: filter_rect.x_min.to_pixels().floor() as i32,
        x_max: filter_rect.x_max.to_pixels().ceil() as i32,
        y_min: filter_rect.y_min.to_pixels().floor() as i32,
        y_max: filter_rect.y_max.to_pixels().ceil() as i32,
    };

    filter_rect
}

fn get_target_image(filter_rect: &UVec2) -> Image {
    let size = Extent3d {
        width: filter_rect.x,
        height: filter_rect.y,
        depth_or_array_layers: 1,
    };

    let mut image = Image::new_uninit(
        size,
        TextureDimension::D2,
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}

fn generate_rectangle_mesh_and_texture_transform() -> Mesh {
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
    )
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));

    mesh
}
