use crate::assets::SwfLoader;
use crate::bundle::FlashAnimation;
use crate::render::FlashRenderPlugin;
use crate::render::material::{BitmapMaterial, GradientMaterial, SwfColorMaterial};

use assets::FlashAnimationSwfData;
use bevy::app::App;
use bevy::asset::Handle;
use bevy::ecs::component::Component;
use bevy::ecs::system::Commands;
use bevy::prelude::{Deref, DerefMut, Entity, Query};

use bevy::render::mesh::Mesh;
use bevy::render::primitives::Aabb;
use bevy::time::Time;
use bevy::{
    app::{Plugin, Update},
    asset::{AssetApp, Assets},
    prelude::{Res, ResMut},
};
use flash_runtime::core::RuntimeInstance;

pub mod assets;
pub mod bundle;
mod render;

pub struct FlashPlugin;

impl Plugin for FlashPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FlashRenderPlugin)
            .init_asset::<FlashAnimationSwfData>()
            .init_asset_loader::<SwfLoader>()
            .add_systems(Update, flash_update);
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct FlashAnimationActiveInstance(Vec<RuntimeInstance>);

fn flash_update(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &FlashAnimation,
        Option<&mut FlashAnimationActiveInstance>,
    )>,
    mut swf_assets: ResMut<Assets<FlashAnimationSwfData>>,
    time: Res<Time>,
) {
    query
        .iter_mut()
        .for_each(|(entity, flash_animation, active_instance)| {
            if let Some(flash) = swf_assets.get_mut(flash_animation.swf.id()) {
                let player = &mut flash.player;
                if let Some(mut active_instance) = active_instance {
                    player.update(&mut active_instance, time.delta_secs());
                } else {
                    let mut active_instance = FlashAnimationActiveInstance::default();
                    // 驱动动画
                    player.update(&mut active_instance, time.delta_secs());
                    commands.entity(entity).insert(active_instance);
                }
            }
        });
}

#[derive(Clone)]
pub enum ShapeDrawType {
    Color(SwfColorMaterial),
    Gradient(GradientMaterial),
    Bitmap(BitmapMaterial),
}

#[derive(Clone)]
pub struct ShapeMesh {
    pub mesh: Handle<Mesh>,
    pub aabb: Aabb,
    pub draw_type: ShapeDrawType,
}
