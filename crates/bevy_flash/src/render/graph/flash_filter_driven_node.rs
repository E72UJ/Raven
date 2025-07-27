use bevy::{
    ecs::{entity::Entity, query::QueryState, world::World},
    render::render_graph::{Node, RenderLabel},
};

use crate::render::intermediate_texture::ExtractedIntermediateTexture;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SingleTextureMultiPassPostProcessingDriverLabel;

pub struct SingleTextureMultiPassPostProcessingDriverNode {
    intermediate_textures: QueryState<(Entity, &'static ExtractedIntermediateTexture)>,
}

impl SingleTextureMultiPassPostProcessingDriverNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            intermediate_textures: world.query(),
        }
    }
}

impl Node for SingleTextureMultiPassPostProcessingDriverNode {
    fn update(&mut self, world: &mut World) {
        self.intermediate_textures.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        _render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w bevy::ecs::world::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        for (entity, intermediate_texture) in self.intermediate_textures.iter_manual(world) {
            graph.run_sub_graph(intermediate_texture.render_graph, vec![], Some(entity))?;
        }
        Ok(())
    }
}
