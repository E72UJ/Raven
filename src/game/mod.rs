// game/mod.rs

use bevy::prelude::*;
use crate::GameScene;

// 基础引用
use bevy::prelude::*;
// 更新时间
use bevy::text::cosmic_text::ttf_parser::Style;
// use bevy_svg::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use bevy::audio::{ AudioPlugin, PlaybackSettings};
use std::path::PathBuf;
// 正确的导入方式
use bevy::{
    color::palettes::basic::*, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    winit::WinitSettings,
    ui::FocusPolicy, // 添加这行
};
use bevy_flash::{FlashPlugin, assets::FlashAnimationSwfData, bundle::FlashAnimation};
use bevy::{audio::Volume, math::ops, prelude::*};
pub const FPS_OVERLAY_Z_INDEX: i32 = i32::MAX - 32;


// 引用

// 按钮组颜色表格
const NORMAL_BUTTON: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// 游戏插件

// 结构体
// / 位置常量
const left_box:f32 = 50.0;

// 点击组件
#[derive(Component)]
struct ClickArea;
// 背景组件标识
#[derive(Component)]
struct Background;


#[derive(Component)]
struct ButtonContainer;
// 添加这些组件定义
#[derive(Component)]
struct DynamicButton;


#[derive(Debug, Deserialize)]
struct Choice {
    text: String,
    goto: String,
}

#[derive(Component)]
struct ClickHandler(String);
// 按钮组颜色表格结束
// 主配置结构体
#[derive(Debug, Deserialize, Resource)]
struct MainConfig {
    title: String,
    assets: AssetPaths,
    settings: GameSettings,
    #[serde(default)]
    global_variables: HashMap<String, String>,
}
// 资源路径结构体
#[derive(Debug, Deserialize)]
struct AssetPaths {
    characters: HashMap<String, String>,
    backgrounds: HashMap<String, String>,
    audio: AudioPaths,
    videos: HashMap<String, String>,
    swf: HashMap<String, String>,
}
// 音频路径结构体
#[derive(Debug, Deserialize)]
struct AudioPaths {
    bgm: HashMap<String, String>,
    sfx: HashMap<String, String>,
    click_sound: String, // 新增音效路径
}
// 游戏设置结构体
#[derive(Debug, Deserialize)]
struct GameSettings {
    initial_scene: String,
    text_speed: u32,
    auto_save: bool,
    resolution: Vec<u32>,
}
// 标签映射资源
// #[derive(Debug, Resource)]
// struct LabelMap(HashMap<String, usize>);
#[derive(Debug, Resource)]
struct LabelMap(HashMap<String, usize>);  // 标签 -> 行索引的映射

// 对话数据结构（支持YAML反序列化）
#[derive(Debug, Deserialize)]
struct Dialogue {
    character: String,
    text: String,
    portrait: String,
    background: Option<String>,  // 新添加的背景字段
    swf: Option<String>, // 新增swf字段
    #[serde(default)] // 如果没有label字段，则为None
    label: Option<String>,
    #[serde(default)] // 如果没有jump字段，则为None
    jump: Option<String>,
    choices: Option<Vec<Choice>>, // 动态的分支选项
}
// 游戏状态资源
#[derive(Debug, Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
    jump_label: Option<String>, // 新增的跳转标签字段
    in_branch_selection: bool, // 新增：是否在分支选择状态
}
// 立绘组件
#[derive(Component)]
struct Portrait;

// 立绘资源句柄
// #[derive(Debug, Resource)]

// 定义音频句柄资源
#[derive(Resource)]
struct ClickSound(Handle<AudioSource>);
#[derive(Resource)]
struct BackClickSound(Handle<AudioSource>);
#[derive(Debug, Resource)] // 添加此行
struct PortraitAssets {
    handles: HashMap<String, Handle<Image>>,
}
// 音频控制
#[derive(Component)]
struct MyMusic;

// 结构体结束
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameScene::Game), setup_game)
            .add_systems(OnExit(GameScene::Game), cleanup_game)
            .add_systems(
                Update,
                (
                    handle_input,
                    output_game_state,
                ).run_if(in_state(GameScene::Game))
            );
    }
}

// 游戏状态组件

// 设置游戏场景
fn setup_game(mut commands: Commands) {
    info!("进入游戏场景");
    
    // 创建游戏状态实体
    // commands.spawn(GameState {
    //     frame_count: 0,
    //     player_position: Vec2::ZERO,
    // });
}

// 清理游戏场景
fn cleanup_game(
    mut commands: Commands,
    entities: Query<Entity, With<GameState>>,
) {
    info!("退出游戏场景");
    
    // 清理游戏状态实体
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}

// 处理输入
fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScene>>,
    mut game_state_query: Query<&mut GameState>,
    time: Res<Time>,
) {
    // 按 ESC 返回菜单
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameScene::Menu);
        return;
    }
    
    // 更新玩家位置
    if let Ok(mut game_state) = game_state_query.get_single_mut() {
        let mut direction = Vec2::ZERO;
        let speed = 100.0;
        
        
    }
}

// 输出游戏状态
fn output_game_state(
    time: Res<Time>,
) {
    println!("成功进入数据")
}