use std::sync::Mutex;

use bevy::{
    color::LinearRgba,
    core_pipeline::{blit::BlitPipeline, upscaling::ViewUpscalingPipeline},
    render::{
        render_graph::ViewNode,
        render_resource::{
            BindGroup, BindGroupEntries, PipelineCache, RenderPassDescriptor, TextureViewId,
        },
        view::ViewTarget,
    },
};

use crate::render::intermediate_texture::ExtractedIntermediateTexture;

#[derive(Default)]
pub struct SingleTextureMultiPassPostProcessingNode {
    cached_texture_bind_group: Mutex<Option<(TextureViewId, BindGroup)>>,
}

impl ViewNode for SingleTextureMultiPassPostProcessingNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewUpscalingPipeline,
        &'static ExtractedIntermediateTexture,
    );

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        (target, upscaling_target, _): bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w bevy::ecs::world::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let pipeline_cache = world.get_resource::<PipelineCache>().unwrap();
        let blit_pipeline = world.get_resource::<BlitPipeline>().unwrap();

        let upscaled_texture = target.main_texture_view();
        let mut cached_bind_group = self.cached_texture_bind_group.lock().unwrap();
        let bind_group = match &mut *cached_bind_group {
            Some((id, bind_group)) if upscaled_texture.id() == *id => bind_group,
            cached_bind_group => {
                let bind_group = render_context.render_device().create_bind_group(
                    None,
                    &blit_pipeline.texture_bind_group,
                    &BindGroupEntries::sequential((upscaled_texture, &blit_pipeline.sampler)),
                );

                let (_, bind_group) = cached_bind_group.insert((upscaled_texture.id(), bind_group));
                bind_group
            }
        };
        let Some(pipeline) = pipeline_cache.get_render_pipeline(upscaling_target.0) else {
            return Ok(());
        };
        let pass_descriptor = RenderPassDescriptor {
            label: Some("upscaling_pass"),
            color_attachments: &[Some(
                target.out_texture_color_attachment(Some(LinearRgba::NONE)),
            )],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut render_pass = render_context
            .command_encoder()
            .begin_render_pass(&pass_descriptor);

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}
