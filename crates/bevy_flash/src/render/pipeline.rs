use std::borrow::Cow;

use bevy::{
    asset::{Handle, weak_handle},
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::{
        resource::Resource,
        system::{Res, SystemState},
        world::FromWorld,
    },
    image::BevyDefault,
    math::Mat4,
    render::{
        mesh::{PrimitiveTopology, VertexBufferLayout},
        render_resource::{
            BindGroupLayout, BindGroupLayoutEntries, BlendState, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState,
            PipelineCache, PolygonMode, PrimitiveState, RenderPipelineDescriptor, Sampler,
            SamplerBindingType, SamplerDescriptor, Shader, ShaderStages, SpecializedRenderPipeline, TextureFormat, TextureSampleType, VertexFormat,
            VertexState, VertexStepMode,
            binding_types::{sampler, texture_2d, uniform_buffer},
        },
        renderer::RenderDevice,
        view::Msaa,
    },
};

use super::{
    graph::filter::{BevelUniform, BlurUniform, ColorMatrixUniform, GlowFilterUniform},
    material::GradientUniforms,
};

pub const INTERMEDIATE_TEXTURE_MESH: Handle<Shader> =
    weak_handle!("f1e2d3c4-b5a6-4978-8c9d-0e1f2a3b4c5d");

pub const INTERMEDIATE_TEXTURE_GRADIENT: Handle<Shader> =
    weak_handle!("a1b2c3d4-e5f6-4789-8a9b-0c1d2e3f4a5b");

#[derive(Resource, Clone)]
pub struct IntermediateTexturePipeline {
    pub view_bind_group_layout: BindGroupLayout,
    pub gradient_bind_group_layout: BindGroupLayout,
    pub bitmap_bind_group_layout: BindGroupLayout,
    pub sampler: Sampler,
}

impl FromWorld for IntermediateTexturePipeline {
    fn from_world(world: &mut bevy::ecs::world::World) -> Self {
        let mut system_state: SystemState<Res<RenderDevice>> = SystemState::new(world);
        let render_device = system_state.get_mut(world);
        let render_device = render_device.into_inner();

        let view_bind_group_layout = render_device.create_bind_group_layout(
            "纹理变换矩阵布局",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX,
                (
                    uniform_buffer::<[[f32; 4]; 4]>(false),
                    uniform_buffer::<Mat4>(false),
                ),
            ),
        );
        let gradient_bind_group_layout = render_device.create_bind_group_layout(
            "渐变纹理布局",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<Mat4>(false),
                    uniform_buffer::<GradientUniforms>(false),
                ),
            ),
        );

        let bitmap_bind_group_layout = render_device.create_bind_group_layout(
            "位图纹理",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::VERTEX_FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<[[f32; 4]; 4]>(false),
                    uniform_buffer::<GradientUniforms>(false),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        IntermediateTexturePipeline {
            view_bind_group_layout,
            gradient_bind_group_layout,
            bitmap_bind_group_layout,
            sampler,
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct IntermediateTextureKey:u8 {
        const NONE     = 0;
        const COLOR    = 1 << 1;
        const GRADIENT = 1 << 2;
        const BITMAP   = 1 << 3;
        const MSAA     = 1 << 4;
    }
}

impl SpecializedRenderPipeline for IntermediateTexturePipeline {
    type Key = IntermediateTextureKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        let vertex_buffer_layout = if key.contains(IntermediateTextureKey::COLOR) {
            VertexBufferLayout::from_vertex_formats(
                VertexStepMode::Vertex,
                vec![VertexFormat::Float32x3, VertexFormat::Float32x4],
            )
        } else {
            VertexBufferLayout::from_vertex_formats(
                VertexStepMode::Vertex,
                vec![VertexFormat::Float32x3],
            )
        };
        if key.contains(IntermediateTextureKey::COLOR) {
            shader_defs.push("VERTEX_COLORS".into());
        }

        let bind_group_layout = if key.contains(IntermediateTextureKey::GRADIENT) {
            vec![
                self.view_bind_group_layout.clone(),
                self.gradient_bind_group_layout.clone(),
            ]
        } else if key.contains(IntermediateTextureKey::BITMAP) {
            vec![
                self.view_bind_group_layout.clone(),
                self.bitmap_bind_group_layout.clone(),
            ]
        } else {
            vec![self.view_bind_group_layout.clone()]
        };

        let shader = if key.contains(IntermediateTextureKey::COLOR) {
            INTERMEDIATE_TEXTURE_MESH
        } else {
            INTERMEDIATE_TEXTURE_GRADIENT
        };

        RenderPipelineDescriptor {
            label: Some(Cow::from("intermediate_render_pipeline")),
            layout: bind_group_layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "vertex".into(),
                buffers: vec![vertex_buffer_layout],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: Msaa::default().samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader,
                shader_defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        }
    }
}

pub const BLUR_FILTER_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("f59e3d1c-7a24-4b8c-82a3-1d94e6f2c705");

pub const COLOR_MATRIX_FILTER_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("1a2b3c4d-5e6f-4789-0123-456789abcdef");

pub const GLOW_FILTER_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("c1d2e3f4-a5b6-4789-0123-456789abcdef");

pub const BEVEL_FILTER_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("e2f8a9d6-3c7b-42f1-8e9d-5a6b4c3d2e1f");
#[derive(Resource)]
pub struct BlurFilterPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for BlurFilterPipeline {
    fn from_world(world: &mut bevy::ecs::world::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let layout = render_device.create_bind_group_layout(
            "blur_filter_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<BlurUniform>(false),
                ),
            ),
        );
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let descriptor = RenderPipelineDescriptor {
            label: Some(Cow::from("blur_filter_render_pipeline")),
            layout: vec![layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: BLUR_FILTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Resource)]
pub struct ColorMatrixFilterPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for ColorMatrixFilterPipeline {
    fn from_world(world: &mut bevy::ecs::world::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let layout = render_device.create_bind_group_layout(
            "color_matrix_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<ColorMatrixUniform>(false),
                ),
            ),
        );
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let descriptor = RenderPipelineDescriptor {
            label: Some(Cow::from("color_matrix_filter_render_pipeline")),
            layout: vec![layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: COLOR_MATRIX_FILTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Resource)]
pub struct GlowFilterPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for GlowFilterPipeline {
    fn from_world(world: &mut bevy::ecs::world::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let layout = render_device.create_bind_group_layout(
            "glow_filter_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<GlowFilterUniform>(false),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                ),
            ),
        );
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let descriptor = RenderPipelineDescriptor {
            label: Some(Cow::from("glow_filter_filter_render_pipeline")),
            layout: vec![layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: GLOW_FILTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Resource)]
pub struct BevelFilterPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for BevelFilterPipeline {
    fn from_world(world: &mut bevy::ecs::world::World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let layout = render_device.create_bind_group_layout(
            "glow_filter_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<BevelUniform>(false),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                ),
            ),
        );
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let descriptor = RenderPipelineDescriptor {
            label: Some(Cow::from("bevel_filter_render_pipeline")),
            layout: vec![layout.clone()],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: BEVEL_FILTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout::from_vertex_formats(
                    VertexStepMode::Vertex,
                    vec![
                        VertexFormat::Float32x2,
                        VertexFormat::Float32x2,
                        VertexFormat::Float32x2,
                        VertexFormat::Float32x2,
                    ],
                )],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: BEVEL_FILTER_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        };

        let pipeline_id = pipeline_cache.queue_render_pipeline(descriptor);

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
