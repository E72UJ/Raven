use bevy::{
    log::warn,
    render::{
        diagnostic::RecordDiagnostics,
        mesh::{RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_graph::ViewNode,
        render_resource::{
            BindGroupEntries, BufferInitDescriptor, BufferUsages, CommandEncoderDescriptor,
            PipelineCache, RenderPassDescriptor,
        },
        sync_world::MainEntity,
        texture::GpuImage,
        view::ViewTarget,
    },
};

use crate::render::{
    RawVertexDrawType, intermediate_texture::ExtractedIntermediateTexture,
    pipeline::IntermediateTexturePipeline,
};

use super::RenderPhases;

#[derive(Default)]
pub struct TextureSynthesisNode;

impl ViewNode for TextureSynthesisNode {
    type ViewQuery = (&'static ExtractedIntermediateTexture, &'static ViewTarget);

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (intermediate_texture, target): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w bevy::ecs::world::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let Some(render_phases) = world.get_resource::<RenderPhases>() else {
            return Ok(());
        };
        let pipeline_cache = world.resource::<PipelineCache>();
        let gpu_images = world.resource::<RenderAssets<GpuImage>>();

        let mesh_allocator = world.resource::<MeshAllocator>();
        let meshes = world.resource::<RenderAssets<RenderMesh>>();

        let view_entity: MainEntity = graph.view_entity().into();
        let Some(render_phase) = render_phases.get(&view_entity) else {
            return Ok(());
        };
        let intermediate_texture_pipeline = world.resource::<IntermediateTexturePipeline>();
        let diagnostics = render_context.diagnostic_recorder();

        let sampler = &intermediate_texture_pipeline.sampler;
        let color_attachments = [Some(target.get_color_attachment())];

        render_context.add_command_buffer_generation_task(move |render_device| {
            let mut command_encoder =
                render_device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("single_texture_multi_pass_command_encoder"),
                });

            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("single_texture_multi_pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // TODO: TrackedRenderPass 怎么使用的？
            // let mut render_pass = TrackedRenderPass::new(&render_device, render_pass);
            let pass_span = diagnostics.pass_span(&mut render_pass, "single_texture_multi_pass");
            if !render_phase.is_empty() {
                for raw_vertex in render_phase.iter() {
                    let (pipeline_id, mesh_draw_type) =
                        (raw_vertex.pipeline_id, &raw_vertex.mesh_draw_type);
                    let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id) else {
                        continue;
                    };

                    let Some(gpu_mesh) = meshes.get(raw_vertex.mesh.id()) else {
                        continue;
                    };
                    let Some(vertex_buffer_slice) =
                        mesh_allocator.mesh_vertex_slice(&raw_vertex.mesh.id())
                    else {
                        continue;
                    };

                    let filter_size = intermediate_texture.filter_size;
                    let delta = filter_size - intermediate_texture.size;
                    let scale = intermediate_texture.scale;
                    let view_matrix = [
                        [2.0 * scale.x / filter_size.x as f32, 0.0, 0.0, 0.0],
                        [0.0, -2.0 * scale.y / filter_size.y as f32, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [
                            -1.0 + (2.0 * (delta.x as f32 / 2.0)) / filter_size.x as f32,
                            1.0 - (2.0 * (delta.y as f32 / 2.0)) / filter_size.y as f32,
                            0.0,
                            1.0,
                        ],
                    ];
                    let view_buffer =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("view_matrix_buffer"),
                            contents: bytemuck::cast_slice(&[view_matrix]),
                            usage: BufferUsages::UNIFORM,
                        });
                    let world_transform =
                        render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some("swf_world_transform_buffer"),
                            contents: bytemuck::cast_slice(&[intermediate_texture.world_transform]),
                            usage: BufferUsages::UNIFORM,
                        });

                    let view_bind_group = render_device.create_bind_group(
                        "view_bind_group",
                        &intermediate_texture_pipeline.view_bind_group_layout,
                        &BindGroupEntries::sequential((
                            view_buffer.as_entire_binding(),
                            world_transform.as_entire_binding(),
                        )),
                    );

                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, &view_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, *vertex_buffer_slice.buffer.slice(..));
                    match mesh_draw_type {
                        RawVertexDrawType::Color => {}
                        RawVertexDrawType::Gradient(gradient) => {
                            let gradient_uniform_buffer =
                                render_device.create_buffer_with_data(&BufferInitDescriptor {
                                    label: Some("渐变缓冲区"),
                                    contents: bytemuck::cast_slice(&[gradient.gradient]),
                                    usage: BufferUsages::UNIFORM,
                                });
                            let texture_transform_buffer =
                                render_device.create_buffer_with_data(&BufferInitDescriptor {
                                    label: Some("渐变纹理变换缓冲区"),
                                    contents: bytemuck::cast_slice(&[gradient.texture_transform]),
                                    usage: BufferUsages::UNIFORM,
                                });
                            let texture_view =
                                &gpu_images.get(&gradient.texture).unwrap().texture_view;
                            let bind_group = render_device.create_bind_group(
                                Some("渐变纹理绑定组"),
                                &intermediate_texture_pipeline.gradient_bind_group_layout,
                                &BindGroupEntries::sequential((
                                    texture_view,
                                    sampler,
                                    texture_transform_buffer.as_entire_binding(),
                                    gradient_uniform_buffer.as_entire_binding(),
                                )),
                            );
                            render_pass.set_bind_group(1, &bind_group, &[]);
                        }
                        _ => {
                            warn!("位图应该不会使用滤镜吧！😊，如果有再做吧！！！")
                        }
                    }
                    match gpu_mesh.buffer_info {
                        RenderMeshBufferInfo::Indexed {
                            count,
                            index_format,
                        } => {
                            let Some(index_buffer_slice) =
                                mesh_allocator.mesh_index_slice(&raw_vertex.mesh.id())
                            else {
                                continue;
                            };
                            render_pass.set_index_buffer(
                                *index_buffer_slice.buffer.slice(..),
                                index_format,
                            );

                            render_pass.draw_indexed(
                                index_buffer_slice.range.start
                                    ..(index_buffer_slice.range.start + count),
                                vertex_buffer_slice.range.start as i32,
                                0..1,
                            );
                        }
                        RenderMeshBufferInfo::NonIndexed => {
                            render_pass.draw(0..gpu_mesh.vertex_count, 0..1);
                        }
                    }
                }
            }

            pass_span.end(&mut render_pass);
            drop(render_pass);
            command_encoder.finish()
        });

        Ok(())
    }
}
