use crate::render::FlashFilters;
use crate::render::intermediate_texture::ExtractedIntermediateTexture;
use crate::render::pipeline::{
    BevelFilterPipeline, BlurFilterPipeline, ColorMatrixFilterPipeline, GlowFilterPipeline,
};
use bevy::math::UVec2;
use bevy::render::render_phase::TrackedRenderPass;
use bevy::render::render_resource::{
    BindGroupEntries, BufferInitDescriptor, BufferUsages, IndexFormat, Operations, PipelineCache,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, ShaderType,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureUsages,
    TextureView, TextureViewDescriptor,
};
use bevy::render::renderer::RenderContext;
use bevy::render::{render_graph::ViewNode, view::ViewTarget};
use bytemuck::{Pod, Zeroable};
use flash_runtime::core::filter::Filter::{BevelFilter, BlurFilter, ColorMatrixFilter, GlowFilter};
#[derive(Default)]
pub struct FlashFilterNode;

impl ViewNode for FlashFilterNode {
    type ViewQuery = (
        &'static ExtractedIntermediateTexture,
        &'static FlashFilters,
        &'static ViewTarget,
    );

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (intermediate_texture, filters, view_target): bevy::ecs::query::QueryItem<
            'w,
            Self::ViewQuery,
        >,
        world: &'w bevy::ecs::world::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let blur_filter_pipeline = world.resource::<BlurFilterPipeline>();
        let color_matrix_filter_pipeline = world.resource::<ColorMatrixFilterPipeline>();
        let glow_filter_pipeline = world.resource::<GlowFilterPipeline>();
        let bevel_filter_pipeline = world.resource::<BevelFilterPipeline>();

        // 以下算法均来自于Ruffle
        for filter in filters.iter() {
            match filter {
                BlurFilter(blur_filter) => {
                    let Some(pipeline) =
                        pipeline_cache.get_render_pipeline(blur_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };
                    apply_blur(
                        &blur_filter,
                        render_context,
                        pipeline,
                        blur_filter_pipeline,
                        view_target,
                        intermediate_texture.filter_size,
                    );
                }
                GlowFilter(glow_filter) => {
                    let Some(blur_filter_render_pipeline) =
                        pipeline_cache.get_render_pipeline(blur_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };
                    let Some(glow_filter_render_pipeline) =
                        pipeline_cache.get_render_pipeline(glow_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };
                    let temp_texture_view = copy_source_texture(render_context, view_target);
                    apply_blur(
                        &glow_filter.inner_blur_filter(),
                        render_context,
                        blur_filter_render_pipeline,
                        blur_filter_pipeline,
                        view_target,
                        intermediate_texture.filter_size,
                    );
                    let post_process = view_target.post_process_write();

                    let render_device = render_context.render_device();
                    let glow_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("glow_filter_bind_group"),
                            contents: bytemuck::cast_slice(&[GlowFilterUniform {
                                color: [
                                    f32::from(glow_filter.color.r) / 255.0,
                                    f32::from(glow_filter.color.g) / 255.0,
                                    f32::from(glow_filter.color.b) / 255.0,
                                    f32::from(glow_filter.color.a) / 255.0,
                                ],
                                strength: glow_filter.strength.to_f32(),
                                inner: if glow_filter.is_inner() { 1 } else { 0 },
                                knockout: if glow_filter.is_knockout() { 1 } else { 0 },
                                composite_source: if glow_filter.composite_source() {
                                    1
                                } else {
                                    0
                                },
                            }]),
                            usage: BufferUsages::UNIFORM,
                        });

                    let bind_group = render_device.create_bind_group(
                        Some("glow_filter_bind_group"),
                        &glow_filter_pipeline.layout,
                        &BindGroupEntries::sequential((
                            &temp_texture_view,
                            &glow_filter_pipeline.sampler,
                            glow_buffer.as_entire_binding(),
                            post_process.source,
                        )),
                    );

                    let mut render_pass = get_render_pass(
                        render_context,
                        post_process.destination,
                        "glow_filter_render_pass",
                    );
                    render_pass.set_render_pipeline(glow_filter_render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.draw(0..3, 0..1);
                }
                ColorMatrixFilter(color_matrix_filter) => {
                    let Some(pipeline) = pipeline_cache
                        .get_render_pipeline(color_matrix_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };
                    let post_process = view_target.post_process_write();
                    let color_matrix_uniform = ColorMatrixUniform {
                        matrix: color_matrix_filter.matrix,
                    };
                    let render_device = render_context.render_device();

                    let color_matrix_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("color_matrix_uniform"),
                            contents: bytemuck::cast_slice(&[color_matrix_uniform]),
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        });

                    let bind_group = render_device.create_bind_group(
                        Some("color_matrix_bind_group"),
                        &color_matrix_filter_pipeline.layout,
                        &BindGroupEntries::sequential((
                            post_process.source,
                            &color_matrix_filter_pipeline.sampler,
                            color_matrix_buffer.as_entire_binding(),
                        )),
                    );
                    let mut render_pass = get_render_pass(
                        render_context,
                        post_process.destination,
                        "color_matrix_filter_render_pass",
                    );
                    // render_context.begin_tracked_render_pass(RenderPassDescriptor {
                    //     label: Some("color_matrix_filter_render_pass"),
                    //     color_attachments: &[Some(RenderPassColorAttachment {
                    //         view: post_process.destination,
                    //         resolve_target: None,
                    //         ops: Operations::default(),
                    //     })],
                    //     depth_stencil_attachment: None,
                    //     timestamp_writes: None,
                    //     occlusion_query_set: None,
                    // });
                    render_pass.set_render_pipeline(pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.draw(0..3, 0..1);
                }
                BevelFilter(bevel_filter) => {
                    let Some(blur_filter_render_pipeline) =
                        pipeline_cache.get_render_pipeline(blur_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };
                    let Some(bevel_filter_render_pipeline) =
                        pipeline_cache.get_render_pipeline(bevel_filter_pipeline.pipeline_id)
                    else {
                        continue;
                    };

                    let temp_texture_view = copy_source_texture(render_context, view_target);
                    apply_blur(
                        &bevel_filter.inner_blur_filter(),
                        render_context,
                        blur_filter_render_pipeline,
                        blur_filter_pipeline,
                        view_target,
                        intermediate_texture.filter_size,
                    );
                    let post_process = view_target.post_process_write();
                    let mut highlight_color = [
                        f32::from(bevel_filter.highlight_color.r) / 255.0,
                        f32::from(bevel_filter.highlight_color.g) / 255.0,
                        f32::from(bevel_filter.highlight_color.b) / 255.0,
                        f32::from(bevel_filter.highlight_color.a) / 255.0,
                    ];
                    highlight_color[0] *= highlight_color[3];
                    highlight_color[1] *= highlight_color[3];
                    highlight_color[2] *= highlight_color[3];
                    let mut shadow_color = [
                        f32::from(bevel_filter.shadow_color.r) / 255.0,
                        f32::from(bevel_filter.shadow_color.g) / 255.0,
                        f32::from(bevel_filter.shadow_color.b) / 255.0,
                        f32::from(bevel_filter.shadow_color.a) / 255.0,
                    ];
                    shadow_color[0] *= shadow_color[3];
                    shadow_color[1] *= shadow_color[3];
                    shadow_color[2] *= shadow_color[3];
                    let bevel_uniform = BevelUniform {
                        highlight_color,
                        shadow_color,
                        strength: bevel_filter.strength.to_f32(),
                        bevel_type: if bevel_filter.is_on_top() {
                            2
                        } else if bevel_filter.is_inner() {
                            1
                        } else {
                            0
                        },
                        knockout: if bevel_filter.is_knockout() { 1 } else { 0 },
                        composite_source: 1,
                    };
                    let distance = bevel_filter.distance.to_f32();
                    let angle = bevel_filter.angle.to_f32();
                    let blur_offset_x = angle.cos() * distance;
                    let blur_offset_y = angle.sin() * distance;
                    let width = intermediate_texture.filter_size.x as f32;
                    let height = intermediate_texture.filter_size.y as f32;
                    let filter_vertex_with_double_blur = vec![
                        FilterVertexWithDoubleBlur {
                            position: [0.0, 0.0],
                            source_uv: [0.0, 0.0],
                            blur_uv_left: [blur_offset_x / width, blur_offset_y / height],
                            blur_uv_right: [
                                (0.0 - blur_offset_x) / width,
                                (0.0 - blur_offset_y) / height,
                            ],
                        },
                        FilterVertexWithDoubleBlur {
                            position: [1.0, 0.0],
                            source_uv: [1.0, 0.0],
                            blur_uv_left: [(width + blur_offset_x) / width, blur_offset_y / height],
                            blur_uv_right: [
                                (width - blur_offset_x) / width,
                                (0.0 - blur_offset_y) / height,
                            ],
                        },
                        FilterVertexWithDoubleBlur {
                            position: [1.0, 1.0],
                            source_uv: [1.0, 1.0],
                            blur_uv_left: [
                                (width + blur_offset_x) / width,
                                (height + blur_offset_y) / height,
                            ],
                            blur_uv_right: [
                                (width - blur_offset_x) / width,
                                (height - blur_offset_y) / height,
                            ],
                        },
                        FilterVertexWithDoubleBlur {
                            position: [0.0, 1.0],
                            source_uv: [0.0, 1.0],
                            blur_uv_left: [
                                blur_offset_x / width,
                                (blur_offset_y + height) / height,
                            ],
                            blur_uv_right: [
                                (0.0 - blur_offset_x) / width,
                                (height - blur_offset_y) / height,
                            ],
                        },
                    ];
                    let render_device = render_context.render_device();
                    let vertex_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("bevel_filter_with_double_uv"),
                            contents: bytemuck::cast_slice(&filter_vertex_with_double_blur),
                            usage: BufferUsages::VERTEX,
                        });
                    let indices = vec![0, 1, 2, 0, 2, 3];
                    let indices_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("bevel_filter_quad_indices"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: BufferUsages::INDEX,
                        });
                    let bevel_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("bevel_filter_buffer"),
                            contents: bytemuck::cast_slice(&[bevel_uniform]),
                            usage: BufferUsages::UNIFORM,
                        });
                    let bind_group = render_device.create_bind_group(
                        "bevel_filter_bind_group",
                        &bevel_filter_pipeline.layout,
                        &BindGroupEntries::sequential((
                            &temp_texture_view,
                            &bevel_filter_pipeline.sampler,
                            bevel_buffer.as_entire_binding(),
                            post_process.source,
                        )),
                    );

                    let mut render_pass = get_render_pass(
                        render_context,
                        post_process.destination,
                        "bevel_filter_render_pass",
                    );
                    render_pass.set_render_pipeline(bevel_filter_render_pipeline);
                    render_pass.set_bind_group(0, &bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(indices_buffer.slice(..), 0, IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                }
                // DropShadowFilter(drop_shadow_filter) => todo!(),
                // ConvolutionFilter(convolution_filter) => todo!(),

                // GradientBevelFilter(gradient_filter) => todo!(),
                // GradientGlowFilter(gradient_filter) => todo!(),
                _ => {}
            }
        }
        Ok(())
    }
}

fn apply_blur<'w>(
    blur_filter: &swf::BlurFilter,
    render_context: &mut RenderContext<'w>,
    pipeline: &RenderPipeline,
    blur_filter_pipeline: &BlurFilterPipeline,
    view_target: &ViewTarget,
    filter_size: UVec2,
) {
    let width = filter_size.x as f32;
    let height = filter_size.y as f32;
    for _ in 0..(blur_filter.num_passes() as usize) {
        for i in 0..2 {
            let post_process = view_target.post_process_write();
            let horizontal = i % 2 == 0;
            let strength = if horizontal {
                blur_filter.blur_x.to_f32()
            } else {
                blur_filter.blur_y.to_f32()
            };
            let full_size = strength.min(255.0);
            if full_size <= 1.0 {
                continue;
            }
            let radius = (full_size - 1.0) / 2.0;
            let m = radius.ceil() - 1.0;
            let alpha = ((radius - m) * 255.0).floor() / 255.0;
            let last_offset = 1.0 / ((1.0 / alpha) + 1.0);
            let last_weight = alpha + 1.0;

            let uniform = BlurUniform {
                direction: if horizontal {
                    [1.0 / width, 0.0]
                } else {
                    [0.0, 1.0 / height]
                },
                full_size,
                m,
                m2: m * 2.0,
                first_weight: alpha,
                last_offset,
                last_weight,
            };
            let render_device = render_context.render_device();
            let blur_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("blur_filter"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: BufferUsages::UNIFORM,
            });

            let bind_group = render_device.create_bind_group(
                Some("blur_filter_bind_group"),
                &blur_filter_pipeline.layout,
                &BindGroupEntries::sequential((
                    post_process.source,
                    &blur_filter_pipeline.sampler,
                    blur_buffer.as_entire_binding(),
                )),
            );
            let mut render_pass = get_render_pass(
                render_context,
                post_process.destination,
                "blur_filter_render_pass",
            );
            // render_context.begin_tracked_render_pass(RenderPassDescriptor {
            //     label: Some("blur_filter_render_pass"),
            //     color_attachments: &[Some(RenderPassColorAttachment {
            //         view: post_process.destination,
            //         resolve_target: None,
            //         ops: Operations::default(),
            //     })],
            //     depth_stencil_attachment: None,
            //     timestamp_writes: None,
            //     occlusion_query_set: None,
            // });
            render_pass.set_render_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

fn copy_source_texture<'a, 'w>(
    render_context: &'a mut RenderContext<'w>,
    view_target: &ViewTarget,
) -> TextureView {
    let source_texture = view_target.post_process_write().source_texture;
    let size = source_texture.size();
    let temp_texture = render_context
        .render_device()
        .create_texture(&TextureDescriptor {
            label: Some("intermediate_texture_id:"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: view_target.main_texture_format(),
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
    render_context.command_encoder().copy_texture_to_texture(
        TexelCopyTextureInfo {
            texture: source_texture,
            mip_level: 0,
            origin: Default::default(),
            aspect: TextureAspect::All,
        },
        TexelCopyTextureInfo {
            texture: &temp_texture,
            mip_level: 0,
            origin: Default::default(),
            aspect: TextureAspect::All,
        },
        size,
    );
    // 还原
    view_target.post_process_write();
    temp_texture.create_view(&TextureViewDescriptor::default())
}

fn get_render_pass<'a, 'w>(
    render_context: &'a mut RenderContext<'w>,
    view: &TextureView,
    label: &str,
) -> TrackedRenderPass<'a> {
    render_context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations::default(),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}

/// 模糊滤镜
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ShaderType, Pod, Zeroable, PartialEq)]
pub struct BlurUniform {
    direction: [f32; 2],
    full_size: f32,
    m: f32,
    m2: f32,
    first_weight: f32,
    last_offset: f32,
    last_weight: f32,
}

/// 颜色矩阵滤镜
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ShaderType, Pod, Zeroable, PartialEq)]
pub struct ColorMatrixUniform {
    pub matrix: [f32; 20],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, ShaderType, Pod, Zeroable, PartialEq)]
pub struct GlowFilterUniform {
    color: [f32; 4],
    strength: f32,
    inner: u32,            // a wasteful bool, but we need to be aligned anyway
    knockout: u32,         // a wasteful bool, but we need to be aligned anyway
    composite_source: u32, // undocumented flash feature, another bool
}

#[repr(C)]
#[derive(Copy, Clone, Debug, ShaderType, Pod, Zeroable, PartialEq)]
pub struct BevelUniform {
    highlight_color: [f32; 4],
    shadow_color: [f32; 4],
    strength: f32,
    bevel_type: u32,       // 0 outer, 1 inner, 2 full
    knockout: u32,         // a wasteful bool, but we need to be aligned anyway
    composite_source: u32, // undocumented flash feature, another bool
}

#[repr(C)]
#[derive(Copy, Clone, Debug, ShaderType, Pod, Zeroable, PartialEq)]
pub struct FilterVertexWithDoubleBlur {
    position: [f32; 2],
    source_uv: [f32; 2],
    blur_uv_left: [f32; 2],
    blur_uv_right: [f32; 2],
}
