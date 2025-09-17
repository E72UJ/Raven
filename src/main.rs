mod game;
mod menu;
mod config;
mod audio;  
mod transition; // 添加模块
mod style;

use bevy::prelude::*;
use menu::MenuPlugin;
use config::{MainConfig, load_main_config};
use crate::game::GamePlugin;
use crate::audio::AudioPlugin;
// use crate::audio::{play_audio, play_audio_with_volume, play_audio_loop};
use crate::transition::{TransitionPlugin, fade_in, fade_out}; //
use style::StylePlugin;
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
}
// fn my_system(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // 播放一次性音效
//     // play_audio(&mut commands, &asset_server, "audio/two.ogg");

//     // // 播放音效并设置音量
//     // play_audio_with_volume(&mut commands, &asset_server, "audio/explosion.ogg", 0.7);

//     // // 循环播放背景音乐
//     play_audio_loop(&mut commands, &asset_server, "audio/5gzps-9i0ey.ogg", 1.0);
// }
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
                // resizable: false,
                // enabled_buttons: bevy::window::EnabledButtons {
                //     maximize: false,
                //     ..Default::default()
                // },
                resolution: (width, height).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(main_config)
        .add_plugins(menu::MenuPlugin)    // 主菜单界面
        .init_state::<GameScene>()
        // .add_systems(Startup, my_system)  
        .add_plugins(StylePlugin)
        .add_plugins(TransitionPlugin)
        .add_plugins(GamePlugin)  
        .add_plugins(AudioPlugin)
        .add_systems(OnEnter(GameScene::Game), (menu_exit_system,))
        .run();
}



