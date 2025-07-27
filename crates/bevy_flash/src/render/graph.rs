use bevy::{
    app::Plugin,
    ecs::resource::Resource,
    platform::collections::hash_map::Entry,
    prelude::{Deref, DerefMut},
    render::{
        RenderApp,
        graph::CameraDriverLabel,
        render_graph::{RenderGraph, RenderGraphApp, RenderLabel, RenderSubGraph, ViewNodeRunner},
        sync_world::{MainEntity, MainEntityHashMap},
    },
    utils::default,
};
use filter::FlashFilterNode;
use flash_filter_driven_node::{
    SingleTextureMultiPassPostProcessingDriverLabel, SingleTextureMultiPassPostProcessingDriverNode,
};
use texture_synthesis::TextureSynthesisNode;
use upscaling::SingleTextureMultiPassPostProcessingNode;

use super::{
    intermediate_texture::SwfRawVertex,
    pipeline::{
        BevelFilterPipeline, BlurFilterPipeline, ColorMatrixFilterPipeline, GlowFilterPipeline,
    },
};

pub(crate) mod filter;
mod flash_filter_driven_node;
mod texture_synthesis;
mod upscaling;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct FlashFilterSubGraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum FlashFilter {
    TextureSynthesis,
    Filter,
    Upscaling,
}

pub struct FlashFilterRenderGraphPlugin;

impl Plugin for FlashFilterRenderGraphPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_sub_graph(FlashFilterSubGraph)
            .add_render_graph_node::<ViewNodeRunner<TextureSynthesisNode>>(
                FlashFilterSubGraph,
                FlashFilter::TextureSynthesis,
            )
            .add_render_graph_node::<ViewNodeRunner<FlashFilterNode>>(
                FlashFilterSubGraph,
                FlashFilter::Filter,
            )
            .add_render_graph_node::<ViewNodeRunner<SingleTextureMultiPassPostProcessingNode>>(
                FlashFilterSubGraph,
                FlashFilter::Upscaling,
            )
            .add_render_graph_edges(
                FlashFilterSubGraph,
                (
                    FlashFilter::TextureSynthesis,
                    FlashFilter::Filter,
                    FlashFilter::Upscaling,
                ),
            );

        let single_texture_multi_pass_post_processing_driver_node =
            SingleTextureMultiPassPostProcessingDriverNode::new(render_app.world_mut());
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(
            SingleTextureMultiPassPostProcessingDriverLabel,
            single_texture_multi_pass_post_processing_driver_node,
        );
        render_graph.add_node_edge(
            SingleTextureMultiPassPostProcessingDriverLabel,
            CameraDriverLabel,
        );
    }

    fn finish(&self, app: &mut bevy::app::App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<BlurFilterPipeline>()
            .init_resource::<ColorMatrixFilterPipeline>()
            .init_resource::<GlowFilterPipeline>()
            .init_resource::<BlurFilterPipeline>()
            .init_resource::<BevelFilterPipeline>();
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderPhases(pub MainEntityHashMap<Vec<SwfRawVertex>>);

impl RenderPhases {
    pub fn insert_or_clear(&mut self, entity: MainEntity) {
        match self.entry(entity) {
            Entry::Occupied(mut entry) => entry.get_mut().clear(),
            Entry::Vacant(entry) => {
                entry.insert(default());
            }
        }
    }
}
