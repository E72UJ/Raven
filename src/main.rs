mod game;
mod menu;
mod config;


use bevy::prelude::*;
use menu::MenuPlugin;
use config::{MainConfig, load_main_config};
use crate::game::GamePlugin;


// 定义游戏场景状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    Game,
    Settings,
}
fn main() {
    
    let main_config = load_main_config();
    let (width, height) = (
        main_config.settings.resolution[0] as f32,
        main_config.settings.resolution[1] as f32,
    );
    
    App::new()
    
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: main_config.title.clone(),
                name: Some("raven.app".into()),
                resizable: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                resolution: (width, height).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(main_config)
        .add_plugins(menu::MenuPlugin)    // 主菜单插件
        .init_state::<GameScene>()
        .add_plugins(GamePlugin)  // 只添加 GamePlugin，移除 MenuPlugin
        .run();
}