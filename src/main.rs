use bevy::prelude::*;

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
};
use bevy_flash::{FlashPlugin, assets::FlashAnimationSwfData, bundle::FlashAnimation};
use bevy::{audio::Volume, math::ops, prelude::*};
pub const FPS_OVERLAY_Z_INDEX: i32 = i32::MAX - 32;


// 按钮组颜色表格

const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// 位置常量
const left_box:f32 = 50.0;

// 背景组件标识
#[derive(Component)]
struct Background;

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
}
// 游戏状态资源
#[derive(Debug, Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
    jump_label: Option<String>, // 新增的跳转标签字段
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
fn main() {
    println!("{:?}", "mac 主程序启动！");
    // 加载主配置
    let main_config = load_main_config();
    let app_window = Some(Window {
        title: main_config.title.clone(),
        name: Some("raven.app".into()),
        resizable: false,
        enabled_buttons: bevy::window::EnabledButtons {
            maximize: false,
            ..Default::default()
        },
        // 从配置文件读取分辨率
        resolution: (
            main_config.settings.resolution[0] as f32,
            main_config.settings.resolution[1] as f32,
        )
            .into(),
        ..default()
    });
    App::new()
        // 载入配置程序
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
        }))
        // 插入插件
        .add_plugins((
            FlashPlugin,
        ))
        .insert_resource(main_config) // 将配置作为资源插入
        // .add_plugins(bevy_svg::prelude::SvgPlugin)
        // 设置背景清除颜色
        // 等效十六进制表示（深蓝紫色）
        // Color::srgb_u8(51, 51, 102)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.4)))
        .add_systems(Startup, (setup_camera, load_portraits, setup_ui,load_audio_resources,load_backgrounds,load_swf_assets))
        .add_systems(Update, (handle_input, update_dialogue, update_portrait,flash_animation,apply_jump,update_background,update_swf))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>,) {
    commands.spawn(Camera2d);

    // commands.spawn(Sprite::from_image(
    //     asset_server.load("background/main.png"),
    // ));
    
    
}

// 加载主配置文件
fn load_main_config() -> MainConfig {
    // let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // let yaml_path = base_path.join("assets/main.yaml");
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let yaml_path3 = exe_dir.join("assets/main.yaml");
    println!("相对的配置路径: {:?}", yaml_path3);
    let yaml_str = fs::read_to_string(yaml_path3).expect("找不到配置文件 assets/main.yaml");

    serde_yaml::from_str(&yaml_str).expect("YAML解析失败，请检查格式")
}
// 从YAML加载对话数据，应用变量替换
fn load_dialogues(config: &MainConfig) -> Vec<Dialogue> {
    // let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    println!("相对的对话路径有: {:?}", exe_dir.join("assets/dialogues.yaml"));
    let yaml_path2 = exe_dir.join("assets/dialogues.yaml");
    let yaml_str = fs::read_to_string(yaml_path2).expect("找不到对话文件 assets/dialogues.yaml");

    // 对YAML字符串进行变量替换
    let mut processed_yaml = yaml_str.clone();

    // 替换全局变量
    for (var_name, var_value) in &config.global_variables {
        processed_yaml = processed_yaml.replace(&format!("${}", var_name), var_value);
    }

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
    commands.spawn(Camera2d);
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
        jump_label: None
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
        println!("{}", path_string);
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
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    // debug_print("var2",&asset_server);
    // 立绘容器
    commands.spawn((
        Name::new("sidebox"),
        
        // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
        // Transform::from_xyz(2.0, 1.0, 0.0),
        // Sprite::sized(Vec2::new(75., 75.)),
        // Transform::from_translation(Vec3::new(-340.0, -100.0, 0.0)),
        // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.4)),
        // Sprite {
        //     image: asset_server.load("characters/protagonist/02.png"),
        //     custom_size: Some(Vec2 { x: 478.4, y: 376.8 }),
        //     ..default()
        // },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            left: Val::Px(0.0),
            top: Val::Px(80.0),
            position_type: PositionType::Absolute,
            
            // align_items: AlignItems::Center,
            // justify_content: JustifyContent::Center,
            ..default()
        },
        // Visibility::Hidden,
        GlobalZIndex(10000),
        ZIndex(1200),
    )).with_children(|parent| {
            // 在这里创建子节点
            parent.spawn((
                Name::new("textbox"),
                ImageNode::new(asset_server.load("characters/protagonist/02.png"),),
                Visibility::Hidden, // 设置为可见
                Transform::from_translation(Vec3::new(1450.0, -750.0, 0.0)).with_scale(Vec3::new(0.5, 0.5, 0.0)),
                
                // Name::new("child_element"),
                // Text::new("子节点文本"),
                // TextFont {
                //     font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                //     font_size: 14.0,
                //     ..default()
                // },
                Node {
                    position_type: PositionType::Relative,
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                // 其他你需要的组件
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
    commands.spawn((
        Name::new("svgload"),
        FlashAnimation {
            // name:"a1",
            swf: asset_server.load("swf/345_c7.swf")
        },
        // Transform::default().with_scale(Vec3::ZERO),
        Visibility::Hidden,
        Transform::from_translation(Vec3::new(-400.0, 240.0, 0.0)).with_scale(Vec3::splat(2.0)),

    ));

    commands.spawn((
        Name::new("spritebox"),
        // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
        Transform::from_xyz(1.0, 1.0, 0.0),
        // Sprite::sized(Vec2::new(75., 75.)),
        Sprite {
            image: asset_server.load("characters/protagonist/default.png"),
            custom_size: Some(Vec2 { x: 400.0, y: 600.0 }),
            ..default()
        },
        Visibility::Hidden,
    ));
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
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(170.0),
                // padding: UiRect::all(Val::Px(30.0)),
                padding: UiRect {
    left: Val::Px(30.0),
    right: Val::Px(30.0),
    top: Val::Px(30.0),
    bottom: Val::Px(30.0),
},
                // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8).into();),
                ..default()
            },
            // 对话框背景颜色
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            // AnimatedText,
        ))
        .with_children(|parent| {
            // 在这里创建子节点
            parent.spawn((
                Name::new("textbox"),
                Text::new("文本框!"),
                // Name::new("child_element"),
                // Text::new("子节点文本"),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                Node {
                    position_type: PositionType::Relative,
                    margin: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                
                // 其他你需要的组件
            ));
        });
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Name::new("namebox"),
        Text::new("戴安娜"),
        // Visibility::Hidden,
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
            left: Val::Px(left_box),
            right: Val::Px(50.0),
            height: Val::Px(50.0),
            width: Val::Px(220.0),
            // padding: UiRect::top(Val::Px(30.0)),
            ..default()
        },
        // 对话框背景颜色
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        GlobalZIndex(2),
        // AnimatedText,
    ));
    // 点击区域
    // 立绘容器
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
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
    mut query: Query<(&Name, &mut Text)>,
) {
    // println!("进入 update_dialogue, 当前行: {}", game_state.current_line);
    
    // 1. 获取当前对话行（如果存在）
    let current_dialogue = if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        dialogue
    } else {
        // 处理结束游戏状态
        for (name, mut text) in &mut query {
            if name.as_str() == "namebox" {
                text.0 = "NULL".to_string();
            }
            if name.as_str() == "textbox" {
                text.0 = "感谢体验，按下ESC退出".to_string();
            }
        }
        println!("对话结束，当前行超出范围");
        return;
    };
    
    // 2. 显示当前对话内容
    for (name, mut text) in &mut query {
        if name.as_str() == "namebox" {
            text.0 = current_dialogue.character.to_string();
        }
        if name.as_str() == "textbox" {
            text.0 = current_dialogue.text.to_string();
        }
    }
    
    // 3. 打印调试信息（在显示之后）
    // println!(
    //     "显示行 {}: 角色='{}', 标签={:?}, 跳转={:?}",
    //     game_state.current_line,
    //     current_dialogue.character,
    //     current_dialogue.label,
    //     current_dialogue.jump
    // );
    
    // 4. 处理跳转逻辑（在显示当前内容之后）
    if let Some(jump_label) = &current_dialogue.jump {
        // std::thread::sleep(std::time::Duration::from_millis(500));
        // println!("检测到跳转指令: {} → '{}'", game_state.current_line, jump_label);
        
        if let Some(&new_line) = label_map.0.get(jump_label) {
            // println!("执行跳转: {} → {}", game_state.current_line, new_line);
            println!(
                "显示行 {}: 角色='{}', 标签={:?}, 跳转={:?}",
                game_state.current_line,
                current_dialogue.character,
                current_dialogue.label,
                current_dialogue.jump
            );
            // game_state.current_line = new_line;
            // game_state.can_go_back = true;
            
            // 递归处理跳转（确保跳转后的内容也能显示）
            // update_dialogue(game_state, label_map, query);
        } else {
            println!("错误: 找不到标签 '{}' 的跳转目标", jump_label);
        }
    }
}

// 输入处理系统
fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    click_sound: Res<ClickSound>, // 引入音频句柄
    back_sound: Res<BackClickSound>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
    // audio: Res<Audio>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    label_map: Res<LabelMap>, // 添加LabelMap资源访问
) {
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Digit0 => game_state.current_line = 0,
            KeyCode::Digit1 => game_state.current_line = 1,
            KeyCode::Digit2 => game_state.current_line = 2,
            _ => {}
        }
    };
    let click = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);

    if click && game_state.current_line < game_state.dialogues.len() {
        let current_dialogue = &game_state.dialogues[game_state.current_line];
        
        // 检查是否有跳转指令
        if let Some(jump_label) = &current_dialogue.jump {
            game_state.jump_label = Some(jump_label.clone());
        } else {
            // 没有跳转指令则前进到下一行
            game_state.current_line += 1;
        }
        
        game_state.can_go_back = true;
        // 播放点击音效
        // play_background_audio("button.ogg")
        play_sound(&click_sound.0,commands);
        // println!("下一个音效触发: {:?}", click_sound.0.id());
            // let sink = music_controller.single();
            // sink.toggle_playback();
        

        // 结束
        
    }
    let back_pressed =
        keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::ArrowLeft);

    if click && game_state.current_line < game_state.dialogues.len() {
        game_state.can_go_back = true; // 前进后可以返回
    }

    // 返回上一页
    if back_pressed && game_state.can_go_back && game_state.current_line > 0 {
        game_state.current_line -= 1;
        // play_sound(&back_sound.0);
        if game_state.current_line == 0 {
            game_state.can_go_back = false; // 回到开始时不能再返回
        }
    }
    // 退出程序
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
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
    // 这里可以根据需要创建一个新的 AudioPlayer 实例并播放音频
    // 你可以在这里设置 PlaybackSettings，也可以选择一次性播放或循环播放
    // 在这里创建音频播放器
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
            Transform::from_translation(Vec3::new(-400.0, 240.0, 0.0)).with_scale(Vec3::splat(2.0)),
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
) {
    // 隐藏所有swf动画
    for (_, mut visibility) in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
    
    // 根据当前对话中的swf字段显示对应动画
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        if let Some(swf_name) = &dialogue.swf {
            // println!("显示SWF动画: {}", swf_name);
            for (name, mut visibility) in query.iter_mut() {
                if name.as_str() == format!("swf_{}", swf_name) {
                    *visibility = Visibility::Visible;
                    break;
                }
            }
        }
    }
}

// 结束swf数据
// 更新背景
fn update_background(
    game_state: Res<GameState>,
    mut query: Query<(&Name, &mut Visibility), With<Background>>,
) {
    // 获取当前对话中的背景信息（如果有的话）
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        // 隐藏所有背景
        for (_, mut visibility) in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        
        // 只有当 dialogue.background 有值时才显示对应背景
        if let Some(bg_name) = &dialogue.background {
            for (name, mut visibility) in query.iter_mut() {
                if name.as_str() == format!("background_{}", bg_name) {
                    *visibility = Visibility::Visible;
                }
            }
        }
        // 如果 dialogue.background 是 None，则所有背景都保持隐藏状态
    }
}