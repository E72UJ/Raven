use bevy::{
    asset::Handle,
    platform::collections::HashMap,
    prelude::{
        Component, Deref, DerefMut, Entity, ReflectComponent, ReflectDefault, Transform, Visibility,
    },
    reflect::Reflect,
};
use swf::CharacterId;

use crate::assets::FlashAnimationSwfData;

///
#[derive(Debug, Clone, Default, DerefMut, Deref)]
pub struct FlashShapeSpawnRecord(HashMap<(CharacterId, usize), Entity>);

impl FlashShapeSpawnRecord {
    pub fn is_generate(&self, id: CharacterId, ref_count: usize) -> bool {
        self.contains_key(&(id, ref_count))
    }

    pub fn cache_entities(&self) -> &HashMap<(CharacterId, usize), Entity> {
        &self
    }

    pub fn get_entity(&self, key: CharacterId, ref_count: usize) -> Option<&Entity> {
        self.get(&(key, ref_count))
    }

    pub fn mark_cached_shape(&mut self, id: CharacterId, ref_count: usize, entity: Entity) {
        self.insert((id, ref_count), entity);
    }
}

#[derive(Default, Component)]
#[require(Transform, Visibility)]
pub struct SwfGraph;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[require(Transform, Visibility)]
#[reflect(Component, Default, Debug)]
pub struct FlashAnimation {
    /// 要渲染的swf资源的引用计数句柄。
    pub swf: Handle<FlashAnimationSwfData>,
}
