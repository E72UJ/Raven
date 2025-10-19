mod audio;
mod config;
mod game;
mod menu;
mod style;
mod toolbar;
mod transition; // 添加模块
mod url;

use crate::game::GamePlugin;
use crate::{audio::AudioPlugin, url::UrlPlugin};
use bevy::prelude::*;
use config::{MainConfig, load_main_config};
use menu::MenuPlugin;
// use crate::audio::{play_audio, play_audio_with_volume, play_audio_loop};
use crate::transition::{TransitionPlugin, fade_in, fade_out}; //
use style::{StylePlugin, on_state_changed}; // 导入 on_state_changed 函数
use toolbar::ToolbarPlugin;

// 定义游戏场景状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[default]
    Menu,
    Game,
    Settings,
    About,
    Help,
    Logo,
    LoadButton,
    GameSettings,
}

fn menu_exit_system(mut commands: Commands) {
    fade_in(&mut commands, 1.6); // 1.0渐入
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
                resizable: true,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: true,
                    ..Default::default()
                },
                resolution: (width as u32, height as u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(main_config)
        .add_plugins(menu::MenuPlugin) // 主菜单界面
        .init_state::<GameScene>()
        .add_plugins(StylePlugin)
        .add_plugins(TransitionPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(UrlPlugin) // 新增的url插件
        .add_plugins(ToolbarPlugin)
        
        // 为每个场景状态变化添加样式更新触发器
        .add_systems(OnEnter(GameScene::Menu), ( on_state_changed))
        .add_systems(OnEnter(GameScene::Game), (menu_exit_system, on_state_changed))
        .add_systems(OnEnter(GameScene::Settings), on_state_changed)
        .add_systems(OnEnter(GameScene::About), on_state_changed)
        .add_systems(OnEnter(GameScene::Help), on_state_changed)

        .add_systems(OnEnter(GameScene::LoadButton), on_state_changed)
        .add_systems(OnEnter(GameScene::GameSettings), on_state_changed)
        
        .run();
}
