mod game;
mod menu;
mod config;


use bevy::prelude::*;
use menu::MenuPlugin;
use config::{MainConfig, load_main_config};


// 定义游戏场景状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    Game,
    Settings,
}
fn main() {
    // 载入配置
    let main_config = MainConfig::load();
   App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: main_config.title.clone(),  // 正确的访问方式
                name: Some("raven.app".into()),
                resizable: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                resolution: (
                    main_config.settings.resolution[0] as f32,
                    main_config.settings.resolution[1] as f32,
                ).into(),
                ..default()  // 简写形式
            }),
            ..default()
        }))
        .insert_resource(main_config)  // 将配置作为资源插入
        .init_state::<GameScene>()
        .add_plugins(MenuPlugin)
        .run();
}