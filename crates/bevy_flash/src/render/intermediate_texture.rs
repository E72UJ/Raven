use std::sync::{Arc, atomic::AtomicUsize};

use bevy::{
    app::Plugin,
    asset::{Asset, Handle, load_internal_asset},
    color::LinearRgba,
    ecs::{
        component::Component,
        entity::Entity,
        schedule::IntoScheduleConfigs,
        system::{Commands, Local, Query, Res, ResMut},
    },
    image::BevyDefault,
    math::{Mat4, UVec2, Vec3},
    platform::collections::{HashMap, HashSet, hash_map::Entry},
    prelude::{Deref, DerefMut, ReflectComponent},
    reflect::{Reflect, TypePath},
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
        camera::{NormalizedRenderTarget, RenderTarget},
        mesh::Mesh,
        render_asset::{self, RenderAssets},
        render_graph::{InternedRenderSubGraph, RenderSubGraph},
        render_resource::{
            CachedRenderPipelineId, Extent3d, PipelineCache, Shader, SpecializedRenderPipelines,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        sync_world::{MainEntity, RenderEntity, SyncToRenderWorld},
        texture::{ColorAttachment, GpuImage, OutputColorAttachment, TextureCache},
        view::{MainTargetTextures, Msaa, ViewTarget, ViewTargetAttachments},
    },
};

use crate::render::pipeline::{
    BEVEL_FILTER_SHADER_HANDLE, BLUR_FILTER_SHADER_HANDLE, COLOR_MATRIX_FILTER_SHADER_HANDLE,
    GLOW_FILTER_SHADER_HANDLE, INTERMEDIATE_TEXTURE_GRADIENT, INTERMEDIATE_TEXTURE_MESH,
};

use super::{
    RawVertexDrawType,
    graph::{FlashFilterSubGraph, RenderPhases},
    pipeline::{IntermediateTextureKey, IntermediateTexturePipeline},
};

pub struct IntermediateTexturePlugin;

impl Plugin for IntermediateTexturePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        load_internal_asset!(
            app,
            INTERMEDIATE_TEXTURE_MESH,
            "shaders/intermediate_texture/color.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            INTERMEDIATE_TEXTURE_GRADIENT,
            "shaders/intermediate_texture/gradient.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            BLUR_FILTER_SHADER_HANDLE,
            "shaders/filters/blur.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            COLOR_MATRIX_FILTER_SHADER_HANDLE,
            "shaders/filters/color_matrix.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            GLOW_FILTER_SHADER_HANDLE,
            "shaders/filters/glow.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            BEVEL_FILTER_SHADER_HANDLE,
            "shaders/filters/bevel.wgsl",
            Shader::from_wgsl
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .init_resource::<RenderPhases>()
            .init_resource::<SpecializedRenderPipelines<IntermediateTexturePipeline>>()
            .add_systems(
                ExtractSchedule,
                (extract_intermediate_texture, extract_intermediate_phase),
            )
            .add_systems(
                Render,
                (
                    specialize_raw_meshes.in_set(RenderSet::PrepareMeshes),
                    prepare_view_attachments
                        .in_set(RenderSet::ManageViews)
                        .before(prepare_intermediate_texture_view_targets),
                    prepare_intermediate_texture_view_targets
                        .in_set(RenderSet::ManageViews)
                        .after(prepare_view_attachments)
                        .after(render_asset::prepare_assets::<GpuImage>),
                    queue_raw_mesh.in_set(RenderSet::Queue),
                ),
            );
    }
}

/// 用于需要进行滤镜处理得到中间纹理
#[derive(Component, Default, Clone)]
#[require(SyncToRenderWorld, FlashFilterRenderGraph::new(FlashFilterSubGraph))]
pub struct IntermediateTexture {
    /// target
    pub target: RenderTarget,
    /// 当前帧是否渲染
    pub is_active: bool,
    /// 全局变换缩放倍数，用于矢量缩放
    pub scale: Vec3,
    /// Graphic 原始bunds大小
    pub size: UVec2,
    /// 应用滤镜后的bunds大小
    pub filter_size: UVec2,
    /// 中间纹理包含的子实体（`Mesh2d`）
    pub view_entities: Vec<SwfRawVertex>,
    /// swf 的world transform 变换，原swf的变换数据由于具有倾斜功能，目前无法使用`bevy`的`Transform`代替
    pub world_transform: Mat4,
}

/// Configures the [`RenderGraph`](crate::render_graph::RenderGraph) name assigned to be run for a given [`IntermediateTexture`] entity.
#[derive(Component, Debug, Deref, DerefMut, Reflect, Clone)]
#[reflect(opaque)]
#[reflect(Component, Debug, Clone)]
pub struct FlashFilterRenderGraph(InternedRenderSubGraph);

impl FlashFilterRenderGraph {
    /// Creates a new [`FlashFilterRenderGraph`] from any string-like type.
    #[inline]
    pub fn new<T: RenderSubGraph>(name: T) -> Self {
        Self(name.intern())
    }
}

pub fn prepare_view_attachments(
    images: Res<RenderAssets<GpuImage>>,
    intermediate_textures: Query<&ExtractedIntermediateTexture>,
    mut view_target_attachments: ResMut<ViewTargetAttachments>,
) {
    for intermediate_texture in intermediate_textures.iter() {
        let Some(target) = &intermediate_texture.target else {
            continue;
        };
        match view_target_attachments.entry(target.clone()) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                if let (Some(texture_view), Some(texture_format)) = match target {
                    bevy::render::camera::NormalizedRenderTarget::Window(_) => {
                        unreachable!("请使用Image作为中间纹理引用")
                    }
                    bevy::render::camera::NormalizedRenderTarget::Image(image_target) => {
                        let gpu_image = images.get(&image_target.handle);
                        (
                            gpu_image.map(|image| &image.texture_view),
                            gpu_image.map(|image| image.texture_format),
                        )
                    }
                    bevy::render::camera::NormalizedRenderTarget::TextureView(_) => {
                        unreachable!("请使用Image作为中间纹理引用")
                    }
                } {
                    entry.insert(OutputColorAttachment::new(
                        texture_view.clone(),
                        texture_format.add_srgb_suffix(),
                    ));
                } else {
                    continue;
                }
            }
        }
    }
}

pub fn prepare_intermediate_texture_view_targets(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    query: Query<(Entity, &ExtractedIntermediateTexture)>,
    view_target_attachments: Res<ViewTargetAttachments>,
) {
    let mut textures = <HashMap<_, _>>::default();
    for (entity, intermediate_texture) in query.iter() {
        let Some(target) = &intermediate_texture.target else {
            continue;
        };
        let Some(out_attachment) = view_target_attachments.get(target) else {
            continue;
        };
        let size = Extent3d {
            width: intermediate_texture.filter_size.x,
            height: intermediate_texture.filter_size.y,
            depth_or_array_layers: 1,
        };
        let main_texture_format = TextureFormat::bevy_default();
        let (a, b, sampled, main_texture) = textures
            .entry((intermediate_texture.filter_size, entity))
            .or_insert_with(|| {
                let descriptor = TextureDescriptor {
                    label: Some("intermediate_texture_id:"),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: main_texture_format,
                    usage: TextureUsages::RENDER_ATTACHMENT
                        | TextureUsages::COPY_SRC
                        | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                };

                let a = texture_cache.get(
                    &render_device,
                    TextureDescriptor {
                        label: Some("main_texture_a"),
                        ..descriptor
                    },
                );
                let b = texture_cache.get(
                    &render_device,
                    TextureDescriptor {
                        label: Some("main_texture_b"),
                        ..descriptor
                    },
                );
                let sampled = if Msaa::default().samples() > 1 {
                    let sampled = texture_cache.get(
                        &render_device,
                        TextureDescriptor {
                            label: Some("main_texture_sampled"),
                            size,
                            mip_level_count: 1,
                            sample_count: Msaa::default().samples(),
                            dimension: TextureDimension::D2,
                            format: main_texture_format,
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            view_formats: descriptor.view_formats,
                        },
                    );
                    Some(sampled)
                } else {
                    None
                };
                let main_texture = Arc::new(AtomicUsize::new(0));
                (a, b, sampled, main_texture)
            });
        let main_textures = MainTargetTextures::new(
            ColorAttachment::new(a.clone(), sampled.clone(), Some(LinearRgba::NONE)),
            ColorAttachment::new(b.clone(), sampled.clone(), Some(LinearRgba::NONE)),
            main_texture.clone(),
        );
        commands.entity(entity).insert(ViewTarget::new(
            main_textures,
            main_texture_format,
            out_attachment.clone(),
        ));
    }
}

#[derive(Component, Clone)]
pub struct ExtractedIntermediateTexture {
    /// target
    pub target: Option<NormalizedRenderTarget>,
    /// 全局变换缩放倍数，用于矢量缩放
    pub scale: Vec3,
    /// Graphic 原始bunds大小
    pub size: UVec2,
    /// 应用滤镜后的bunds大小
    pub filter_size: UVec2,
    /// 中间纹理包含的子实体（`Mesh2d`）
    pub view_entities: Vec<SwfRawVertex>,
    /// swf 的world transform 变换，保证矢量缩放、旋转。
    pub world_transform: Mat4,
    pub render_graph: InternedRenderSubGraph,
}

pub fn extract_intermediate_texture(
    mut commands: Commands,
    intermediate_textures: Extract<
        Query<(RenderEntity, &IntermediateTexture, &FlashFilterRenderGraph)>,
    >,
) {
    for (render_entity, intermediate_texture, render_graph) in intermediate_textures.iter() {
        if !intermediate_texture.is_active {
            commands
                .entity(render_entity)
                .remove::<ExtractedIntermediateTexture>();
            continue;
        }
        commands
            .entity(render_entity)
            .insert(ExtractedIntermediateTexture {
                target: intermediate_texture.target.normalize(None),
                scale: intermediate_texture.scale,
                size: intermediate_texture.size,
                filter_size: intermediate_texture.filter_size,
                view_entities: intermediate_texture.view_entities.clone(),
                world_transform: intermediate_texture.world_transform,
                render_graph: render_graph.0,
            });
    }
}

#[derive(Clone, Asset, TypePath)]
pub struct SwfRawVertex {
    pub mesh: Handle<Mesh>,
    pub mesh_draw_type: RawVertexDrawType,
    pub pipeline_id: CachedRenderPipelineId,
}

pub fn extract_intermediate_phase(
    query: Extract<Query<(RenderEntity, &IntermediateTexture)>>,
    mut render_phases: ResMut<RenderPhases>,
    mut live_entities: Local<HashSet<MainEntity>>,
) {
    live_entities.clear();
    for (main_entity, intermediate_texture) in query.iter() {
        if !intermediate_texture.is_active {
            continue;
        }
        render_phases.insert_or_clear(main_entity.into());
        live_entities.insert(main_entity.into());
    }
    render_phases.retain(|entity, _| live_entities.contains(entity));
}

pub fn specialize_raw_meshes(
    mut specialized_render_pipelines: ResMut<
        SpecializedRenderPipelines<IntermediateTexturePipeline>,
    >,
    intermediate_texture_pipeline: Res<IntermediateTexturePipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut query: Query<&mut ExtractedIntermediateTexture>,
) {
    for mut intermediate_texture in query.iter_mut() {
        for raw_vertex in intermediate_texture.view_entities.iter_mut() {
            let key = match raw_vertex.mesh_draw_type {
                RawVertexDrawType::Color => IntermediateTextureKey::COLOR,
                RawVertexDrawType::Gradient(_) => IntermediateTextureKey::GRADIENT,
                RawVertexDrawType::Bitmap => IntermediateTextureKey::BITMAP,
            };
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &intermediate_texture_pipeline,
                key,
            );
            raw_vertex.pipeline_id = pipeline_id;
        }
    }
}

pub fn queue_raw_mesh(
    mut query: Query<(Entity, &mut ExtractedIntermediateTexture)>,
    mut render_phases: ResMut<RenderPhases>,
) {
    for (entity, mut intermediate_texture) in query.iter_mut() {
        let main_entity: MainEntity = entity.into();
        let Some(render_phase) = render_phases.get_mut(&main_entity) else {
            continue;
        };
        render_phase.append(&mut intermediate_texture.view_entities);
    }
}
