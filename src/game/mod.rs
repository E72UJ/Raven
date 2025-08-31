// game/mod.rs
use bevy::prelude::*;
use crate::audio;
use crate::style;
use crate::GameScene;
use std::time::Duration;
use std::fmt::Debug;
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


// 包调用
use crate::config::{MainConfig, load_main_config};

use crate::transition::{fade_in, fade_out};

use Raven::dissolve::{RenpyDissolve, RenpyDissolvePlugin, RenpyDissolveTransition};

use Raven::typewriter;
use typewriter::{TypewriterText, typewriter_system, TypewriterPlugin};

use crate::style::UiStyleSheet;
use crate::style::StylePlugin;

use crate::audio::{play_audio, play_audio_with_volume, play_audio_loop};

// 包调用结束

// 引用

// 按钮组颜色表格
// const NORMAL_BUTTON: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct Option1Button;

const NORMAL_BUTTON: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.0);
// 游戏插件

// 阻塞系统
#[derive(Debug, Resource)]
struct BlockState {
    blocked: bool,
    blocked_line: Option<usize>,
    unblock_condition: Option<UnblockCondition>,
}

#[derive(Debug)]
enum UnblockCondition {
    Click,          // 点击后解除阻塞
    KeyPress(KeyCode), // 按下特定键后解除
    Timer(Duration), // 定时器到期后解除
}

// 阻塞系统结束
// 立绘组件
#[derive(Component)]
struct FadeAnimation {
    timer: Timer,
    start_alpha: f32,
    end_alpha: f32,
}

#[derive(Component)]
struct AnimationTarget;
// 立绘组件结束
// 打字机组件
#[derive(Component)]
struct CurrentText;

#[derive(Component)]
struct Typewriter {
    full_text: String,           // 完整文本
    current_index: usize,        // 当前显示到第几个字符
    timer: Timer,                // 控制打字速度的计时器
    is_finished: bool,           // 是否完成打字
}
// #[derive(Component)]
// struct TypewriterText {
//     full_text: String,
//     current_length: usize,
//     timer: Timer,
// }

impl Typewriter {
    fn new(text: String, chars_per_second: f32) -> Self {
        let delay = Duration::from_secs_f32(1.0 / chars_per_second);
        Self {
            full_text: text,
            current_index: 0,
            timer: Timer::new(delay, TimerMode::Repeating),
            is_finished: false,
        }
    }
    
    fn get_current_text(&self) -> String {
        if self.current_index >= self.full_text.len() {
            return self.full_text.clone();
        }
        
        // 正确处理UTF-8字符
        self.full_text.chars().take(self.current_index).collect()
    }
}
// 结构体
// / 位置常量
const left_box:f32 = 80.0;

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

#[derive(Component)]
struct ButtonImages {
    normal: Handle<Image>,
    hovered: Handle<Image>,
    pressed: Handle<Image>,
}
// 按钮组颜色表格结束
// 主配置结构体
// #[derive(Debug, Deserialize, Resource)]
// struct MainConfig {
//     title: String,
//     assets: AssetPaths,
//     settings: GameSettings,
//     #[serde(default)]
//     global_variables: HashMap<String, String>,
// }
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
    #[serde(default)]
    bgm: Option<String>,
    background: Option<String>,  // 新添加的背景字段
    swf: Option<String>, // 新增swf字段
    #[serde(default)] // 如果没有label字段，则为None
    label: Option<String>,
    #[serde(default)] // 如果没有jump字段，则为None
    jump: Option<String>,
    choices: Option<Vec<Choice>>, // 动态的分支选项
    #[serde(default)] // 如果没有pause字段，则为None
    pause: Option<bool>, //
}
// 游戏状态资源
#[derive(Debug, Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
    jump_label: Option<String>, // 新增的跳转标签字段
    in_branch_selection: bool, // 新增：是否在分支选择状态
    is_blocked: bool, // 是否被阻塞
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


// 主函数
#[derive(Resource, Default)]
pub struct CurrentAudio {
    pub current_bgm: Option<String>,
    pub current_entity: Option<Entity>, // 添加这一行跟踪音频实体
}
// 结构体结束
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FlashPlugin>() {
            app.add_plugins(FlashPlugin);

        }
        app
            // 只在启动时加载资源，不创建UI
            .add_systems(Startup, (
                load_main_config_system,
                setup_camera,
                load_portraits,
                load_audio_resources,
                load_backgrounds,
                // load_swf_assets,
                // setup_ui,  // 移除这行！
                

            ).chain())
            // 进入游戏场景时才创建UI和游戏状态
            .add_systems(OnEnter(GameScene::Game), (
                setup_game_state,
                setup_ui,  // 移到这里
                load_swf_assets,
       
            ).chain())
            .add_plugins(RenpyDissolvePlugin)
            // .add_plugins(StylePlugin)  
            .insert_resource(CurrentAudio::default()) 
            // .add_plugins(TypewriterPlugin)
            .add_systems(OnExit(GameScene::Game), cleanup_game)
            .add_systems(
                Update,
                (
                    handle_input,
                    // debug_flash_position,
                    create_dynamic_buttons.after(handle_input), // 确保输入处理在按钮创建之前
                    output_game_state,
                    update_dialogue, 
                    update_audio,
                    // typewriter_system.after(update_dialogue),
                    update_portrait,
                    flash_animation.run_if(in_state(GameScene::Game)),
                    apply_jump,
                    update_background,
                    update_swf.run_if(in_state(GameScene::Game)),
                    keyboard_system,
                    handle_choice_buttons,
                    create_dynamic_buttons.run_if(should_create_buttons),
                    button_interaction_system,

                    button_image_system,
                    update_typewriter
                    // fade_animation_system
                ).run_if(in_state(GameScene::Game))
            );
    }
}

// 游戏状态组件

// 设置游戏场景
// 将配置加载作为独立的系统
fn load_main_config_system(mut commands: Commands) {
    let main_config = load_main_config();
    // println!("{}",main_config.settings.font.clone());
    commands.insert_resource(main_config);
}

// 简化的游戏状态设置
fn setup_game_state(mut commands: Commands, config: Res<MainConfig>,asset_server: Res<AssetServer>) {
    // info!("进入游戏场景");
    commands.spawn(Camera2d);



    let dialogues: Vec<Dialogue> = load_dialogues(&config);
    
    // 创建标签映射
    let mut label_map = HashMap::new();
    for (index, dialogue) in dialogues.iter().enumerate() {
        if let Some(label) = dialogue.label.as_ref() {
            label_map.insert(label.clone(), index);
        }
    }
    
    commands.insert_resource(GameState {
        current_line: 0,
        dialogues,
        can_go_back: false,
        jump_label: None,
        in_branch_selection: false,
        is_blocked: false
    });
    
    commands.insert_resource(LabelMap(label_map));
}

// 清理游戏场景

// 函数库
fn load_dialogues(config: &MainConfig) -> Vec<Dialogue> {
    // let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    println!("相对的对话路径有: {:?}", exe_dir.join("assets/dialogues.yaml"));
    let yaml_path2 = exe_dir.join("assets/dialogues.yaml");
    let yaml_str = fs::read_to_string(yaml_path2).expect("找不到对话文件 assets/dialogues.yaml");

    // 对YAML字符串进行变量替换
    let mut processed_yaml = yaml_str.clone();

    // 替换全局变量
    // for (var_name, var_value) in &config.global_variables {
    //     processed_yaml = processed_yaml.replace(&format!("${}", var_name), var_value);
    // }

    // 替换标题
    processed_yaml = processed_yaml.replace("$title", &config.title);

    // 替换资源路径（简化处理）
    // 背景图片替换
    for (bg_name, bg_path) in &config.assets.backgrounds {
        processed_yaml = processed_yaml.replace(&format!("$backgrounds.{}", bg_name), bg_path);
    }

    // 音频替换
    for (bgm_name, bgm_path) in &config.assets.audio.bgm {
        processed_yaml = processed_yaml.replace(&format!("$audio.bgm.{}", bgm_name), bgm_path);
    }

    for (sfx_name, sfx_path) in &config.assets.audio.sfx {
        processed_yaml = processed_yaml.replace(&format!("$audio.sfx.{}", sfx_name), sfx_path);
    }

    // 角色立绘替换
    for (char_name, char_path) in &config.assets.characters {
        processed_yaml = processed_yaml.replace(&format!("$characters.{}", char_name), char_path);
    }
    // debug_print("var4",&processed_yaml);
    serde_yaml::from_str(&processed_yaml).expect("YAML解析失败，请检查格式")
}
// 初始化游戏的状态
fn setup_camera(mut commands: Commands, config: Res<MainConfig>) {

    // commands.spawn((
    //     Camera2d,
    //     Transform::default(),
    //     // 移除自定义的Camera配置
    // ));

    let dialogues: Vec<Dialogue> = load_dialogues(&config);
    // 创建映射代码
    // 创建标签映射
    let mut label_map = HashMap::new();
    for (index, dialogue) in dialogues.iter().enumerate() {
        if let Some(label) = dialogue.label.as_ref() {  // 使用 as_ref() 获取引用
            label_map.insert(label.clone(), index);
        }
    }
    commands.insert_resource(GameState {
        current_line: 0,
        dialogues: load_dialogues(&config),
        can_go_back: false, // 初始时不能返回
        jump_label: None,
        in_branch_selection: false,
        is_blocked:false
    });
    // println!("label_map: {:?}", label_map[1].jump);
    commands.insert_resource(LabelMap(label_map));
        // 插入标签映射资源
}
// 加载立绘资源 - 使用标准库的Path和PathBuf修改后的版本
fn load_portraits(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    let mut portrait_assets = PortraitAssets {
        handles: HashMap::new(),
    };

    // 遍历配置文件中的所有角色
    for (character_name, character_path) in &config.assets.characters {
        // 确保路径不包含重复的assets前缀
        let character_path = if character_path.starts_with("assets/") {
            character_path.trim_start_matches("assets/").to_string()
        } else {
            character_path.clone()
        };

        // 使用正斜杠来确保路径格式一致
        let path_string = format!("{}/default.png", character_path.replace('\\', "/"));
        // println!("{}", path_string);
        let handle = asset_server.load(&path_string);
        portrait_assets
            .handles
            .insert(character_name.clone(), handle);
    }
    println!("=== 所有立绘路径 ===");
    for character_name in portrait_assets.handles.keys() {
        println!("角色: {}", character_name);
    }

    println!("==================");
    // println!("{}",portrait_assets);
    commands.insert_resource(portrait_assets);
}
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>,stylesheet: Res<UiStyleSheet>,) {
    let mut click_area_entity = commands
        .spawn((
            Name::new("click_area"),
            // Button, // 添加这行
            ClickArea,
            Node {
                width: Val::Px(1400.0),     // 固定宽度800像素
                height: Val::Px(700.0),    // 固定高度600像素
                bottom: Val::Px(80.0),
                left: Val::Px(0.0),  // 添加左边定位
                position_type: PositionType::Absolute,
                
                ..default()
            },
            BackgroundColor(Color::NONE), // 完全透明
            GlobalZIndex(9999),
            Interaction::default(), 
            // Button,
            FocusPolicy::Pass, // 关键：让焦点穿透
            Visibility::Visible,
        ))
        .with_children(|parent| {

                });
// 分支创建============
commands.spawn((
        Name::new("choice_container"),
        ButtonContainer,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(90.0), // 在对话框上方
            height: Val::Px(150.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(1000),
        Visibility::Visible, // 初始隐藏
    ));

// 分支创建结束===============
// sidebox
    commands.spawn((
        Name::new("sidebox"),
        ImageNode::new(asset_server.load("characters/protagonist/02.png"),),
        // Transform::from_translation(Vec3::new(1450.0, -750.0, 0.0))
        // .with_scale(Vec3::new(0.5, 0.5, 0.0)),                
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(-10.0),    // 1450 * 0.2
            top: Val::Px(166.0),     // (80 + 750) * 0.2
            width: Val::Px(578.4),   // 2892 * 0.2
            height: Val::Px(476.8),  // 2384 * 0.2
            ..default()
        },

        Visibility::Hidden,
        // BackgroundColor(Color::srgba(0.4, 0.4, 0.1, 0.4)),
        GlobalZIndex(10000),
        ZIndex(1200),
    )).with_children(|parent| {
            // 在这里创建子节点
            parent.spawn((
                Name::new("textbox"),

                Visibility::Visible,
                Node {
                    position_type: PositionType::Relative,
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
               
            ));
        });
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        // BackgroundColor(Color::srgba(0.4, 0.4, 0.1, 1.0)),
        Portrait,
    ));
// 交互按钮2
        // commands.spawn((
        //     Button,
        //     Node {
        //         position_type: PositionType::Absolute,
        //         top: Val::Px(300.0),
        //         left: Val::Px(120.0),
        //         width: Val::Px(300.0),
        //         height: Val::Px(220.0),
        //         border: UiRect::all(Val::Px(2.0)),
        //         justify_content: JustifyContent::Center,
        //         align_items: AlignItems::Center,
        //         ..default()
        //     },
        //     Visibility::Hidden,
        //     BorderColor(Color::BLACK),
        //     BorderRadius::all(Val::Px(1.0)),
        //     BackgroundColor(NORMAL_BUTTON),
        //     GlobalZIndex(10000),
            
        //     Option1Button,
        // )).with_children(|parent| {
        //     // Text作为子节点
        //     parent.spawn((
        //         Text::new("选项1"),
        //         TextFont {
        //             font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
        //             font_size: 20.0,
        //             ..default()
        //         },
        //         TextColor(Color::WHITE),
        //     ));
        // });

    commands.spawn((
        Name::new("spritebox"),
        // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
        Transform::from_xyz(0.0, -24.0, 0.0),
        // Sprite::sized(Vec2::new(75., 75.)),
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            image: asset_server.load("characters/protagonist/default.png"),
            // custom_size: Some(Vec2 { x: 1400.0, y: 770.0 }),
            ..default()
        },
        Visibility::Hidden,
        RenpyDissolve::fade_in(2.5), // 使用渐入效果
    ));

    // commands.spawn((
    //     Name::new("one"),
    //     // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
    //     Transform::from_xyz(0.0, -24.0, 0.0),
    //     // Sprite::sized(Vec2::new(75., 75.)),
    //     Sprite {
    //         color: Color::srgba(1.0, 1.0, 1.0, 0.0),
    //         image: asset_server.load("fps/6.png"),
    //         // custom_size: Some(Vec2 { x: 1400.0, y: 770.0 }),
    //         ..default()
    //     },
    //     Visibility::Visible,
    //     RenpyDissolve::fade_in(2.5), // 使用渐入效果
    // ));
//     commands.spawn((
//     Name::new("spritebox2"),
//     Transform::from_xyz(0.0, -24.0, 0.0),
//     Sprite {
//         color: Color::srgba(1.0, 0.0, 0.0, 0.0), // 红色，初始完全透明
//         custom_size: Some(Vec2 { x: 400.0, y: 600.0 }),
//         ..default()
//     },
//     // 不需要 image 字段，就是纯色
//     Visibility::Visible,
//     RenpyDissolve::fade_in(2.0),
// ));
    // commands.spawn((
    //     Name::new("background"),
    //     // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
    //     Transform::from_xyz(1.0, 2.0, 0.0),
    //     // Sprite::sized(Vec2::new(75., 75.)),
    //     Sprite {
    //         image: asset_server.load("background/one.png"),
    //         // custom_size: Some(Vec2 { x: 1200.0, y: 660.0 }),
    //         ..default()
    //     },
    //     // Visibility::Hidden,
    // ));
    let dialog_padding = stylesheet.get_padding("styles","dialog_box");
    let dialog_pos = stylesheet.get_position("styles","dialog_box");
    let main_config = load_main_config();
    let menu_bg = stylesheet.get_background_color("menu", "menu_box");
    commands
        .spawn((
            
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            // Name::new("textbox"),
            // Text::new("文本框!"),
            // TextFont {
            //     font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            //     font_size:28.0,
            //     ..default()
            // },
            // TextShadow::default(),
            // TextLayout::new_with_justify(JustifyText::Left),
            Name::new("text"),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                // bottom: Val::Px(0.0),
                top: dialog_pos.top,
                bottom: dialog_pos.bottom,
                left: dialog_pos.left,
                right: dialog_pos.right,
                // width: Val::Px(1080.0),
                
                height: Val::Px(170.0),
                // padding: UiRect::all(Val::Px(30.0)),
                padding: UiRect {
                    left: dialog_padding.left,
                    right: dialog_padding.right,
                    top: dialog_padding.top,
                    bottom: dialog_padding.bottom,
                },
                // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8).into();),
                ..default()
                
            },
            // 对话框背景颜色
            ImageNode::new(asset_server.load("gui/textbox3.png")),
            // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            // AnimatedText,
        ))
        .with_children(|parent| {
            // 在这里创建子节点
            parent.spawn((
                Name::new("textbox"),
                Text::new("文本框!"),
                TextFont {
                    // font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font: asset_server.load(main_config.settings.font.clone()),
                    font_size: stylesheet.get_font_size("styles","textbox"),
                    
                    ..default()
                },
                TextColor(stylesheet.get_text_color("styles","textbox")),

                Node {
                    position_type: PositionType::Relative,
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                
                // 其他你需要的组件
                // CurrentText,
                TypewriterText {
                    full_text: "".to_string(),
                    current_length: 0,
                    timer: Timer::from_seconds(0.02, TimerMode::Repeating), // 每50毫秒显示一个字符
                    is_active: true,  // 添加这一行
                },
            ));
        });
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Name::new("namebox"),
        Text::new("戴安娜"),
        Visibility::Visible,
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 28.0,
            line_height: bevy::text::LineHeight::Px(50.),
            ..default()
        },
        
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
        // TextColor(Color::srgba(0.6, 0.1, 0.1, 0.8)),
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(230.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(50.0),
            width: Val::Px(220.0),
            // padding: UiRect::top(Val::Px(30.0)),
            ..default()
        },
        // BackgroundColor(Color::NONE),
        // 对话框背景颜色
        // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        GlobalZIndex(2),
        ImageNode::new(asset_server.load("gui/textbox2.png")),

        // AnimatedText,
    ));
    // 点击区域
    // 立绘容器
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            // bottom: Val::Px(-10.0),
            ..default()
        },
        // BackgroundColor(Color::srgba(0.4, 0.4, 0.1, 1.0)),
        GlobalZIndex(5),
        // Portrait,
    ));
}

// 更新对话文本
fn update_dialogue(
    mut game_state: ResMut<GameState>,
    label_map: Res<LabelMap>,
    stylesheet: Res<UiStyleSheet>,
    mut dialog_query: Query<(&Name, &mut Visibility, &mut Node), Without<Text>>, // 查询对话框容器
    mut query: Query<(&Name, &mut Text, &mut Visibility, Option<&mut TextColor>)>,
    // mut typewriter_query: Query<(&mut Text, &mut TypewriterText)>,  // 查询同时拥有Text和TypewriterText组件的实体
    
) {

    // println!("进入 update_dialogue, 当前行: {}", game_state.current_line);

    // println!("  menu 样式读取: {:?}", stylesheet.get_font_size("menu","menu_box"));
    // stylesheet.debug_print();
    // 1. 获取当前对话行（如果存在）

    let current_dialogue = if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        dialogue
    } else {
        // 处理结束游戏状态
        for (name, mut text, mut visibility, text_color) in &mut query {
            if name.as_str() == "namebox" {
                text.0 = "NULL".to_string();
                *visibility = Visibility::Hidden; // 隐藏 namebox
            }
            if name.as_str() == "textbox" {
                text.0 = "感谢体验，按下ESC退出".to_string();
            }
        }
        println!("对话结束，当前行超出范围");
        return;
    };
    
    // 2. 显示当前对话内容
    for (name, mut text, mut visibility, text_color) in &mut query {
        if name.as_str() == "namebox" {
            if current_dialogue.character == "none" {
                *visibility = Visibility::Hidden; // 如果 character 为 "none", 隐藏 namebox
            } else {
                *visibility = Visibility::Visible;
                text.0 = current_dialogue.character.to_string();
                
                // 根据角色名称设置不同颜色
                if let Some(mut color) = text_color {
                    match current_dialogue.character.as_str() {
                        "希尔薇" => color.0 = Color::srgb(0.761, 1.0, 0.8), // 粉红色
                        "我" => color.0 = Color::srgb(0.3, 0.7, 1.0),     // 蓝色
                        "艾莉娅" => color.0 = Color::srgb(0.8, 0.6, 1.0), // 紫色
                        "莉莉" => color.0 = Color::srgb(1.0, 0.8, 0.3),   // 金色
                        _ => color.0 = Color::WHITE,                      // 默认白色
                    }
                }
            }
        }

        if name.as_str() == "textbox" {
            // if name.as_str() == "text" {

            // }

                text.0 = current_dialogue.text.to_string();
                // println!("{}",current_dialogue.text.to_string());
        }
    }
     // 查找对话框容器
    for (name, mut visibility, mut node) in dialog_query.iter_mut() {
        if name.as_str() == "text" {
            // 处理对话框显示/隐藏逻辑
             if current_dialogue.text == "none" {
                    *visibility = Visibility::Hidden; // 如果 character 为 "none", 隐藏 namebox
                } else {
                    *visibility = Visibility::Visible;
                }
        }
    }
    if let Some(jump_label) = &current_dialogue.jump {
        if let Some(&new_line) = label_map.0.get(jump_label) {
            println!(
                "显示行 {}: 角色='{}', 标签={:?}, 跳转={:?}",
                game_state.current_line,
                current_dialogue.character,
                current_dialogue.label,
                current_dialogue.jump
            );
        } else {
            println!("错误: 找不到标签 '{}' 的跳转目标", jump_label);
        }
    }
}


fn handle_input(
    mut interaction_query: Query<(&Interaction, &Name), (Changed<Interaction>, With<Node>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    back_sound: Res<BackClickSound>,
    label_map: Res<LabelMap>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
    mut commands: Commands, 
    config: Res<MainConfig>,

) {

    println!("===============");
   // 检查是否被阻塞
    let is_paused = if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        match dialogue.pause {
            Some(true) => {
                println!("当前行需要暂停交互");
                true
            }
            Some(false) => {
                println!("这一行不需要暂停");
                false
            }
            None => {
                println!("pause 字段为 None，默认不暂停");
                false
            }
        }
    } else {
        false
    };
    
    // 检查是否有按钮点击（可以解除阻塞）
    let mut button_clicked = false;
    for (interaction, name) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // 检查是否是选择按钮或其他可交互按钮
            if name.as_str().starts_with("choice_") || 
               name.as_str() == "Option1Button" ||
               name.as_str() == "click_area" {
                button_clicked = true;
                println!("检测到按钮点击: {}", name.as_str());
                break;
            }
        }
    }
    
    // 如果点击了按钮，解除阻塞
    if button_clicked && is_paused {
        println!("按钮点击，解除阻塞");
        // 不设置 is_blocked，让按钮系统处理具体逻辑
        return; // 让按钮系统处理点击
    }
    
    // 如果被阻塞且没有按钮点击，阻止输入处理
    if is_paused && !button_clicked {
        println!("输入被阻塞，等待按钮交互");
        return;
    }
    
    println!("============");
    
    // ESC键始终可用
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // 数字键快速跳转（始终可用）
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Digit0 => game_state.current_line = 0,
            KeyCode::Digit1 => game_state.current_line = 1,
            KeyCode::Digit2 => game_state.current_line = 2,
            _ => {}
        }
    }

    // println!("数据测试 {}",config.settings.rewind);
// 返回上一页（根据配置决定是否可用）
let back_pressed = keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::ArrowLeft);

    for key in keys.get_just_pressed() {
        println!("handle_input 检测到按键: {:?}", key);
    }
if back_pressed && config.settings.rewind && game_state.can_go_back && game_state.current_line > 0 {

    game_state.current_line -= 1;
    play_sound(&back_sound.0, commands.reborrow());
    
    // 只有在到达第一行时才禁用回退
    if game_state.current_line == 0 {
        game_state.can_go_back = false;
    }
}else{
    if config.settings.rewind == false {
        game_state.can_go_back = false;
    }
}

    // 如果在分支选择状态，禁用前进操作
   if game_state.in_branch_selection {
        println!("🚫 输入被阻塞 - 当前在分支选择状态");
        println!("   当前行: {}", game_state.current_line);
        if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
            if let Some(choices) = &dialogue.choices {
                println!("   可用选项: {}", choices.len());
                for (i, choice) in choices.iter().enumerate() {
                    println!("     {}. {}", i + 1, choice.text);
                }
            }
        }
        return; // 这里就是问题所在！
    }

    // 检测前进输入（键盘 + 鼠标 + 点击区域）
    let keyboard_click = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
    let mouse_click = mouse.just_pressed(MouseButton::Left);
    
    // 检查点击区域
    let mut click_area_pressed = false;
    for (interaction, name) in &interaction_query {
        if *interaction == Interaction::Pressed && name.as_str() == "click_area" {
            click_area_pressed = true;
            println!("test 点击了透明区域");
            break;
        }
    }

    // 统一处理前进逻辑
    // let should_advance = keyboard_click || mouse_click || click_area_pressed;
    let should_advance = click_area_pressed || keyboard_click;
    if should_advance && game_state.current_line < game_state.dialogues.len() {
        let current_dialogue = &game_state.dialogues[game_state.current_line];
        
        // 检查是否有跳转指令
        if let Some(jump_label) = &current_dialogue.jump {
            game_state.jump_label = Some(jump_label.clone());
        } else {
            // 没有跳转指令则前进到下一行
            game_state.current_line += 1;
        }
        
        game_state.can_go_back = true;
        play_sound(&back_sound.0, commands.reborrow());
    }
}
// fn update_portrait(
//     game_state: Res<GameState>,
//     portraits: Res<PortraitAssets>,
//     mut query: Query<(&mut Sprite, &Name, &mut Visibility)>, //
// ) {
//     // 遍历所有实体，检查名称
//     for (mut sprite, name, mut visibility) in query.iter_mut() {
//         if name.as_str() == "spritebox" {
//             // 检查当前是否有对话
//             if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
//                 match portraits.handles.get(&dialogue.portrait) {
//                     Some(handle) => {
//                         // 仅更新 image 字段，保留其他字段
//                         sprite.image = handle.clone();
//                         *visibility = Visibility::Visible;
//                     }
//                     None => {
//                         *visibility = Visibility::Hidden;
//                     }
//                 }
//             } else {
//                 *visibility = Visibility::Hidden;
//             }
//         }
//     }
// }
fn update_portrait(
    game_state: Res<GameState>,
    portraits: Res<PortraitAssets>,
    mut query: Query<(&mut Sprite, &mut Name, &mut Visibility)>,
) {
    // 先保存查询结果到变量
    // 遍历所有实体，检查名称
    for (mut texture_handle, name, mut visibility) in query.iter_mut() {
        if name.as_str() == "spritebox" {
            // 检查当前是否有对话
            if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
                // println!("数据测试,{}",game_state.current_line);
                // 从资源映射中获取立绘路径
                match portraits.handles.get(&dialogue.portrait) {
                    Some(handle) => {
                        // println!("{:?}", "=============");
                        // println!("{:?}", texture_handle);
                        // custom_size:Some(Vec2 { x: 400.0, y: 600.0 });
                        // println!("{:?}", "=============");
                        // 更新纹理并显示
                        texture_handle.image = handle.clone();
                        *visibility = Visibility::Visible;
                    }
                    None => {
                        // 找不到立绘时隐藏
                        *visibility = Visibility::Hidden;
                        eprintln!("找不到立绘资源: {}", dialogue.portrait);
                    }
                }
            } else {
                // 没有对话时隐藏
                *visibility = Visibility::Hidden;
            }
        }
    }
    // 输出 Debug 格式内容
    // println!("GameState = {:?}", *game_state);
    // println!("query = {:?}", query);
    // 后续操作（例如修改 image 和 visibility）

    // println!("{?}","thisi my home");
}
// 专有调试函数
fn debug_print<T: std::fmt::Debug>(label: &str, value: &T) {
    println!("{} = {:#?}", label, value);
}
// 专有工具函数，返回程序所读取的目录
// pub fn get_executable_directory() -> Result<String, Box<dyn std::error::Error>> {
//     // 获取当前可执行文件路径
//     let mut path = env::current_exe()?;

//     // 移除可执行文件名，保留目录路径
//     path.pop();

//     // 将路径转换为字符串（自动处理非法UTF-8字符）
//     Ok(path.to_string_lossy().into_owned())

// }
fn get_current_working_dir_absolute() -> String {
    env::current_dir() // 直接返回绝对路径
        .expect("Failed to get current directory")
        .to_str()
        .expect("Path is not valid UTF-8")
        .to_string()
}
// fn svgload(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     let svg = asset_server.load("characters/svg/long.svg");
//     commands.spawn((
//         Svg2d(svg),
//         Origin::Center, // Origin::TopLeft is the default
//         Transform {
//             scale: Vec3::new(1.0, 1.0, 1.0),
//             ..Default::default()
//         }
//     ));
// }
// 动画控制
fn flash_animation(
    mut flashes: ResMut<Assets<FlashAnimationSwfData>>,
    mut flash_swf_data_events: EventReader<AssetEvent<FlashAnimationSwfData>>,
) -> Result {
    for event in flash_swf_data_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let flash = flashes.get_mut(*id).unwrap();
            flash.player.set_on_completion(Box::new(|player| {
                player.set_play_animation("default", false).unwrap();
            }));

            flash.player.set_play_animation("default", true)?;
        }
    }
    Ok(())
}
// 音效加载系统
// 在初始化时加载音效
fn load_audio_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    let click_sound_handle: Handle<AudioSource> = asset_server.load(&config.assets.audio.click_sound);
    let backclick_sound_handle: Handle<AudioSource> = asset_server.load(&config.assets.audio.click_sound);
    // let click_sound_handle = asset_server.load("button.ogg");
    // 将向下页面的音效启动
    commands.insert_resource(ClickSound(click_sound_handle));
    commands.insert_resource(BackClickSound(backclick_sound_handle));
}
// fn play_background_audio(
//     asset_server: Res<AssetServer>, 
//     mut commands: Commands
// ) {
//     commands.spawn((
//         AudioPlayer::new(asset_server.load("button.ogg")),
//         // PlaybackSettings::ONCE,
//     ));
// }
// 播放音效的函数
fn play_sound(audio_handle: &Handle<AudioSource>,mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(audio_handle.clone()),
        PlaybackSettings::ONCE,
    ));
}
fn apply_jump(
    label_map: Res<LabelMap>,
    mut game_state: ResMut<GameState>,
) {
    if let Some(jump_label) = game_state.jump_label.take() {
        if let Some(&target_line) = label_map.0.get(&jump_label) {
            println!("执行跳转: {} → {}", game_state.current_line, target_line);
            game_state.current_line = target_line;
            game_state.can_go_back = true;
        } else {
            eprintln!("错误: 找不到标签 '{}' 的跳转目标", jump_label);
        }
    }
}

// 预加载系统
// fn preload_sounds(asset_server: Res<AssetServer>) {
//     asset_server.load::<AudioSource>("button.ogg");
// }
// 背景加载系统
fn load_backgrounds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    // 遍历配置文件中的所有背景
    for (bg_name, bg_path) in &config.assets.backgrounds {
        commands.spawn((
            Name::new(format!("background_{}", bg_name)),
            Background, // 添加背景组件标识
            Sprite {
                image: asset_server.load(bg_path),
                custom_size: Some(Vec2::new(1400.0, 770.0)), 
                ..default()

                
            },
            Transform::from_xyz(0.0, 0.0, -10.0), // 设置在较低的z层
            Visibility::Hidden, // 默认隐藏，需要时显示
        ));
    }
    
    println!("=== 已加载背景 ===");
    for bg_name in config.assets.backgrounds.keys() {
        println!("背景: {}", bg_name);
    }
    println!("==================");
}
// 更新swf数据
// 新增swf资源预加载系统
fn load_swf_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    println!("=== 加载SWF资源 ===");
    println!("配置中的swf数量: {}", config.assets.swf.len());
    
    for (swf_name, swf_path) in &config.assets.swf {
        println!("正在加载SWF: {} -> {}", swf_name, swf_path);
        
        let swf_handle = asset_server.load(swf_path);
        println!("SWF句柄创建成功: {:?}", swf_handle);
        
        commands.spawn((
            Name::new(format!("swf_{}", swf_name)),
            FlashAnimation {
                swf: swf_handle
            },
            Transform::from_translation(Vec3::new(200.0,100.0, 0.0)).with_scale(Vec3::splat(1.0)),
            Visibility::Hidden,
        ));
        
        println!("SWF实体已生成: swf_{}", swf_name);
    }
    println!("==================");
}
// 新增swf更新系统
// 修改swf更新系统

fn update_swf(
    game_state: Res<GameState>,
    mut query: Query<(&Name, &mut Visibility), With<FlashAnimation>>,
    flashes: Res<Assets<FlashAnimationSwfData>>, // 添加资源检查
    flash_query: Query<&FlashAnimation>, // 添加Flash组件查询
) {
    //    println!("=== update_swf 调试信息 ===");
    // println!("查询到的SWF实体数量: {}", query.iter().count());
    
    for (name, visibility) in query.iter() {
        // println!("发现实体: {}, 当前可见性: {:?}", name.as_str(), *visibility);
    }


    for (_, mut visibility) in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    
    // 根据当前对话中的swf字段显示对应动画
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        if let Some(swf_name) = &dialogue.swf {
            let target_name = format!("swf_{}", swf_name);
            // println!("尝试显示SWF动画: {} (查找实体: {})", swf_name, target_name);
            
            let mut found = false;
            
            // 遍历所有Flash实体寻找匹配的名称
            for (name, mut visibility) in query.iter_mut() {
                if name.as_str() == target_name {
                    // 检查对应的SWF资源是否已加载
                    let mut resource_loaded = false;
                    
                    // 检查资源加载状态
                    for flash_animation in flash_query.iter() {
                        if let Some(flash_data) = flashes.get(&flash_animation.swf) {
                            resource_loaded = true;
                            break;
                        }
                    }
                    
                    if resource_loaded {
                        *visibility = Visibility::Visible;
                        // println!("✓ 成功显示SWF: {}", target_name);
                        found = true;
                        break;
                    } else {
                        println!("⚠ SWF资源尚未加载完成: {}", target_name);
                    }
                }
            }
            
            if !found {
                println!("✗ 未找到SWF实体: {}", target_name);
                println!("可用的Flash实体:");
                for (name, _) in query.iter() {
                    println!("  - {}", name.as_str());
                }
            }
        } else {
            // println!("当前对话没有SWF字段");
        }
        
        if game_state.is_changed() {
            // println!("==================");
        }
    }
}

// 结束swf数据
// 更新背景
fn update_background(
    game_state: Res<GameState>,
    mut query: Query<(&Name, &mut Visibility), With<Background>>,
    mut commands: Commands,
) {
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        if let Some(new_bg_name) = &dialogue.background {
            let target_bg = format!("background_{}", new_bg_name);

            // 检查当前是否已经显示了这个背景
            let mut current_visible = None;
            let mut target_exists = false;

            for (name, visibility) in query.iter() {
                if *visibility == Visibility::Visible {
                    current_visible = Some(name.as_str());
                }
                if name.as_str() == target_bg {
                    target_exists = true;
                }
            }

            // 如果目标背景存在且与当前背景不同，执行渐变切换
            if target_exists && current_visible.as_ref() != Some(&target_bg.as_str()) {
                println!("切换背景: {:?} -> {}", current_visible, target_bg);

                // 直接调用你的渐变函数
                fade_in(&mut commands, 0.8);
                
                // 更新背景可见性
                for (name, mut visibility) in query.iter_mut() {
                    if name.as_str() == target_bg {
                        *visibility = Visibility::Visible;
                    } else {
                        *visibility = Visibility::Hidden;
                    }
                }
            }
        } else {
            // 没有背景时，隐藏所有背景
            for (_, mut visibility) in query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
fn keyboard_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    back_click_sound: Res<BackClickSound>,
    mut commands: Commands,
) {
    // 向左箭头键（回退）在分支选择状态下仍然可用
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        if game_state.can_go_back && game_state.current_line > 0 {
            game_state.current_line -= 1;
            // play_sound(&back_click_sound.0, commands);
            
            if game_state.current_line == 0 {
                game_state.can_go_back = false;
            }
            
            // 如果回退导致离开了分支选择的位置，退出分支选择状态
            // 这里你可以根据具体逻辑调整
            if game_state.in_branch_selection && game_state.current_line < 5 { // 假设第5行是分支选择
                game_state.in_branch_selection = false;
            }
            
            println!("回退到第 {} 行", game_state.current_line);
        }
    }
}
fn button_interaction_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Name,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color, name) in &mut interaction_query {
        // 透明点击区域特殊处理
        if name.as_str() == "click_area" {
            *color = Color::NONE.into();
            border_color.0 = Color::NONE;
            continue;
        }

        // 所有其他按钮（包括动态按钮）的统一处理
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgba(0.1, 0.1, 0.1, 0.8);
                println!("按下了按钮: {}", name.as_str());
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
                println!("悬停在按钮: {}", name.as_str());
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn create_dynamic_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_state: ResMut<GameState>,
    existing_buttons: Query<Entity, With<DynamicButton>>,
    button_container: Query<Entity, With<ButtonContainer>>,
    mut click_area_query: Query<&mut Visibility, With<ClickArea>>,
) {
    let current_line = game_state.current_line;
    
    // 先检查是否有对话和选择，但不借用
    let has_dialogue = game_state.dialogues.get(current_line).is_some();
    let has_choices = game_state.dialogues.get(current_line)
        .and_then(|d| d.choices.as_ref())
        .map(|choices| choices.len() > 0)
        .unwrap_or(false);
    
    if has_dialogue {
        if has_choices {
            // 现在可以安全修改 game_state
            game_state.in_branch_selection = true;
            // println!("{}",game_state.in_branch_selection);
            
            // 隐藏点击区域
            if let Ok(mut visibility) = click_area_query.get_single_mut() {
                *visibility = Visibility::Hidden;
            }
            
            // 清除现有按钮
            for entity in existing_buttons.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
    
            if let Some(dialogue) = game_state.dialogues.get(current_line) {
                if let Some(choices) = &dialogue.choices {
                    println!("发现 {} 个选择分支", choices.len());
                    
                    if let Ok(container) = button_container.get_single() {
                        for (index, choice) in choices.iter().enumerate() {
                            // 创建按钮的代码...
                            commands.entity(container).with_children(|parent| {
                                parent.spawn((
                                    Button,
                                    DynamicButton,
                                    ClickHandler(choice.goto.to_string()),
                                    Interaction::default(),
                                    Name::new(format!("choice_{}", index)),
                                    // 你的按钮样式代码...
                                    Node {
                                        position_type: PositionType::Relative,
                                        bottom: Val::Px(100.0),
                                        top: Val::Px(-220.0),
                                        left: Val::Px(320.0),
                                        width: Val::Px(700.0),
                                        height: Val::Px(40.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect {
                                            left: Val::Px(2.0),
                                            right: Val::Px(2.0),
                                            top: Val::Px(5.0),
                                            bottom: Val::Px(5.0),
                                        },
                                        ..default()
                                    },
                                     ImageNode::new(asset_server.load("gui/choice_idle_background2.png")),
                                    ButtonImages {
                                        normal: asset_server.load("gui/choice_idle_background2.png"),
                                        hovered: asset_server.load("gui/choice_hover_background2.png"),
                                        pressed: asset_server.load("gui/choice_hover_background2.png"),
                                    },
                                    // BackgroundColor(NORMAL_BUTTON),
                                    // BorderColor(Color::BLACK),
                                    // BorderRadius::all(Val::Px(5.0)),
                                    
                                    Visibility::Visible,
                                )).with_children(|button| {
                                    button.spawn((
                                        Text::new(choice.text.clone()),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 17.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            });
                        }
                    }
                }
            }
        } else {
            // 没有选择分支
            game_state.in_branch_selection = false;
            
            if let Ok(mut visibility) = click_area_query.get_single_mut() {
                *visibility = Visibility::Visible;
            }
            
            for entity in existing_buttons.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
// fn should_create_buttons(
//     game_state: Res<GameState>,
//     existing_buttons: Query<(), With<DynamicButton>>,
// ) -> bool {
//     let current_line = game_state.current_line;
    
//     // 检查当前行是否有选择分支
//     let has_choices = game_state.dialogues.get(current_line)
//         .and_then(|d| d.choices.as_ref())
//         .map(|choices| !choices.is_empty())
//         .unwrap_or(false);
    
//     // 检查是否已经有按钮存在
//     let buttons_exist = !existing_buttons.is_empty();
    
//     // 只在需要创建按钮但还没有按钮，或者需要清除按钮但还有按钮时运行
//     (has_choices && !buttons_exist) || (!has_choices && buttons_exist)
// }

fn handle_choice_buttons(
    mut interaction_query: Query<(&Interaction, &ClickHandler), (Changed<Interaction>, With<DynamicButton>)>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    mut commands: Commands,
) {
    for (interaction, click_handler) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // play_sound(&click_sound.0, commands);
            println!("===============分支按钮被按下=");
            // 解析跳转目标
            if let Ok(goto_line) = click_handler.0.parse::<usize>() {
                game_state.current_line = goto_line;
                game_state.can_go_back = true;
                game_state.in_branch_selection = false;
                println!("跳转到第 {} 行", goto_line);
            }
        }
    }
}

// 函数库
fn cleanup_game(
    mut commands: Commands,
    // 查询所有需要清理的实体
    game_entities: Query<Entity, Or<(
        With<Portrait>,
        With<Background>, 
        With<ClickArea>,
        With<ButtonContainer>,
        With<DynamicButton>,
        With<FlashAnimation>,
    )>>,
    // 查询文本实体
    text_entities: Query<Entity, (With<Text>, With<Name>)>,
    // 查询所有带有特定名称的实体
    named_entities: Query<(Entity, &Name)>,
) {
    info!("清理游戏场景");
    
    // 清理游戏相关的实体
    for entity in game_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // 清理特定名称的实体
    for (entity, name) in named_entities.iter() {
        match name.as_str() {
            "textbox" | "namebox" | "sidebox" | "spritebox" | 
            "click_area" | "choice_container" | "svgload" => {
                commands.entity(entity).despawn_recursive();
            }
            _ if name.as_str().starts_with("background_") => {
                commands.entity(entity).despawn_recursive();
            }
            _ if name.as_str().starts_with("swf_") => {
                commands.entity(entity).despawn_recursive();
            }
            _ if name.as_str().starts_with("choice_") => {
                commands.entity(entity).despawn_recursive();
            }
            _ => {}
        }
    }
    
    // 移除游戏状态资源
    commands.remove_resource::<GameState>();
    commands.remove_resource::<LabelMap>();
    commands.remove_resource::<PortraitAssets>();
    
    info!("游戏场景清理完成");
}

// 处理输入


// 输出游戏状态
fn output_game_state(
    time: Res<Time>,
) {
    // println!("成功进入数据")
}

fn should_create_buttons(
    game_state: Res<GameState>,
    existing_buttons: Query<(), With<DynamicButton>>,
) -> bool {
    let current_line = game_state.current_line;
    
    // 检查当前行是否有选择分支
    let has_choices = game_state.dialogues.get(current_line)
        .and_then(|d| d.choices.as_ref())
        .map(|choices| !choices.is_empty())
        .unwrap_or(false);
    
    // 检查是否已经有按钮存在
    let buttons_exist = !existing_buttons.is_empty();
    
    // 只在需要创建按钮但还没有按钮，或者需要清除按钮但还有按钮时运行
    (has_choices && !buttons_exist) || (!has_choices && buttons_exist)
}

// 条件检查函数
fn any_swf_visible(
    query: Query<&Visibility, With<FlashAnimation>>,
) -> bool {
    query.iter().any(|visibility| *visibility == Visibility::Visible)
}

// 检查swf 的摄像机事业
// 检查Flash实体的Transform
// fn debug_flash_position(
//     query: Query<(&Name, &Transform, &Visibility), With<FlashAnimation>>,
// ) {
//     for (name, transform, visibility) in query.iter() {
//         println!("Flash {}: pos={:?}, visible={:?}", 
//                 name, transform.translation, visibility);
//     }
// }

// flash显示控制器
// fn setup_minimal_swf(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     println!("=== 创建最小SWF系统 ===");
    
//     // 硬编码创建一个SWF实体
//     commands.spawn((
//         Name::new("test_swf"),
//         FlashAnimation {
//             swf: asset_server.load("swf/66.swf")  // 硬编码路径
//         },
//         Transform::from_translation(Vec3::new(-400.0, 0.0, 0.0)).with_scale(Vec3::splat(2.0)),
//         Visibility::Visible,  // 直接设置为可见
//     ));
    
//     println!("SWF实体已创建: test_swf");
//     println!("路径: swf/66.swf");
//     println!("位置: (0, 0, 0)");
//     println!("缩放: 1.0");
//     println!("==================");
// }
fn button_image_system(
    mut query: Query<
        (&Interaction, &mut ImageNode, &ButtonImages), 
        (Changed<Interaction>, With<Button>)
    >,
) {
    for (interaction, mut image_node, button_images) in &mut query {
        image_node.image = match *interaction {
            Interaction::Pressed => button_images.pressed.clone(),
            Interaction::Hovered => button_images.hovered.clone(),
            Interaction::None => button_images.normal.clone(),
        };
    }
}

fn menu_exit_system(mut commands: Commands) {
    fade_in(&mut commands, 1.6); // 1.0渐入
}
// fn fade_animation_system(
//     time: Res<Time>,
//     mut query: Query<(Entity, &mut FadeAnimation, &mut Sprite), With<AnimationTarget>>,
//     mut commands: Commands,
// ) {
//     for (entity, mut fade_anim, mut sprite) in query.iter_mut() {
//         fade_anim.timer.tick(time.delta());
        
//         if !fade_anim.timer.finished() {
//             let progress = fade_anim.timer.elapsed_secs() / fade_anim.timer.duration().as_secs_f32();
            
//             // 使用 Ren'Py 风格的缓动
//             let eased_progress = ren_py_dissolve(progress);
            
//             let current_alpha = fade_anim.start_alpha + (fade_anim.end_alpha - fade_anim.start_alpha) * eased_progress;
            
//             // 增加一些平滑处理
//             let smoothed_alpha = (current_alpha * 255.0).round() / 255.0;
//             sprite.color.set_alpha(smoothed_alpha);
//         } else {
//             sprite.color.set_alpha(fade_anim.end_alpha);
//             commands.entity(entity).remove::<FadeAnimation>();
//         }
//     }
// }

// 缓动函数 - 超级平滑的渐入效果
fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0_f32.powf(-10.0 * t)
    }
}
fn ren_py_dissolve(t: f32) -> f32 {
    // Ren'Py 实际使用的是一个分段的平滑曲线
    if t < 0.1 {
        // 前10%非常缓慢
        t * t * 5.0
    } else if t < 0.8 {
        // 中间70%线性但稍有加速
        0.05 + (t - 0.1) * 1.2
    } else {
        // 最后20%快速完成
        0.89 + (t - 0.8) * 0.55 * (2.0 - t)
    }
}
// 可选的其他缓动函数
fn ease_out_sine(t: f32) -> f32 {
    (t * std::f32::consts::PI / 2.0).sin()
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// 函数测试
fn update_typewriter(
     mut query: Query<(&mut Text, &mut TypewriterText)>,  // 查询同时拥有Text和TypewriterText组件的实体
) {
for (mut text, mut typewriter) in query.iter_mut() {
    if typewriter.is_active {  // 这里 typewriter 是 Mut<TypewriterText> 而不是 TypewriterText
        // println!("{}",typewriter.is_active);
    }
}
}


// fn update_sidebox(
//     game_state: Res<GameState>,
//     mut query: Query<(&Name, &mut Visibility, &mut Node), With<ImageNode>>,
// ) {
//     if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
//         for (name, mut visibility, mut node) in query.iter_mut() {
//             if name.as_str() == "sidebox" {
//                 // 根据对话数据决定是否显示
//                 if dialogue.show_character {
//                     *visibility = Visibility::Visible;
                    
//                     // 动态调整位置
//                     if dialogue.character_position == "left" {
//                         node.left = Val::Px(-10.0);
//                     } else if dialogue.character_position == "right" {
//                         node.left = Val::Px(1200.0);
//                     }
                    
//                     // 动态调整大小
//                     match dialogue.character_size {
//                         "small" => {
//                             node.width = Val::Px(289.2);   // 0.1倍
//                             node.height = Val::Px(238.4);
//                         },
//                         "medium" => {
//                             node.width = Val::Px(578.4);   // 0.2倍
//                             node.height = Val::Px(476.8);
//                         },
//                         "large" => {
//                             node.width = Val::Px(1446.0);  // 0.5倍
//                             node.height = Val::Px(1192.0);
//                         },
//                         _ => {}
//                     }
//                 } else {
//                     *visibility = Visibility::Hidden;
//                 }
//                 break;
//             }
//         }
//     }
// }
// fn update_audio(
//     game_state: Res<GameState>,
//     mut query: Query<(&Name, &mut Visibility), With<Background>>,
//     mut commands: Commands,
//     asset_server: Res<AssetServer>
// ) {
//     if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
//         if let Some(bgm) = &dialogue.bgm {
//             let target_name = bgm;
//             println!("{}",target_name);
//             play_audio(&mut commands, &asset_server, &target_name);

//         }
//     }
    
    
// }
fn update_audio(
    game_state: Res<GameState>,
    mut current_audio: ResMut<CurrentAudio>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        if let Some(bgm) = &dialogue.bgm {
            let target_bgm = bgm.clone();
            let current_playing = current_audio.current_bgm.as_ref();
            let bgm_changed = current_playing != Some(&target_bgm);

            if bgm_changed {
                println!("切换BGM: {:?} -> {}", current_playing, target_bgm);

                // 停止当前播放的音频实体
                if let Some(entity) = current_audio.current_entity {
                    commands.entity(entity).despawn();
                }

                // 播放新BGM并获取实体
                let audio_path = format!("audio/{}", bgm);
                let new_entity = play_audio(&mut commands, &asset_server, &audio_path);
                
                // 更新状态
                current_audio.current_bgm = Some(target_bgm);
                current_audio.current_entity = Some(new_entity);
            }
        } else {
            // 没有BGM时，停止当前播放
            if let Some(entity) = current_audio.current_entity {
                commands.entity(entity).despawn();
                current_audio.current_bgm = None;
                current_audio.current_entity = None;
            }
        }
    }
}

// 交互区域系统
fn on_hover_enter(
    trigger: Trigger<Pointer<Over>>, 
    mut sprites: Query<&mut Sprite>,
) {
    if let Ok(mut sprite) = sprites.get_mut(trigger.target) { // 使用 trigger.target
        sprite.color = Color::srgb(0.0, 1.0, 1.0);
        
        println!("鼠标悬停进入！");
        
    }
}

fn recolor_on<E: Debug + Clone + Reflect>(color: Color) -> impl Fn(Trigger<E>, Query<&mut Sprite>) {
    move |ev, mut sprites| {
        let Ok(mut sprite) = sprites.get_mut(ev.target()) else {
            return;
        };
        sprite.color = color;
    }
}

fn handle_option1_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Option1Button>)>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    mut commands: Commands,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            println!("Option1按钮被点击！");
            
            // 播放音效
            // play_sound(&click_sound.0, commands.reborrow());
            
            // 解除阻塞并前进
            // game_state.is_blocked = false;
            
            // 根据你的逻辑决定如何处理

        }
    }
}