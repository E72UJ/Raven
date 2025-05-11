#![allow(unused_variables, unused_mut, unused_assignments)]
use bevy::prelude::*;
use serde::Deserialize;
use std::{fs, path};
use std::collections::HashMap;
use std::path::PathBuf;
pub const FPS_OVERLAY_ZINDEX: i32 = i32::MAX - 32;

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
}
// 游戏设置结构体
#[derive(Debug, Deserialize)]
struct GameSettings {
    initial_scene: String,
    text_speed: u32,
    auto_save: bool,
    resolution: Vec<u32>,
}

// 对话数据结构（支持YAML反序列化）
#[derive(Debug, Deserialize)]
struct Dialogue {
    character: String,
    text: String,
    portrait: String,
}
// 游戏状态资源
#[derive(Debug,Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
}
// 立绘组件
#[derive(Component)]
struct Portrait;

// 立绘资源句柄
#[derive(Debug,Resource)]
struct PortraitAssets {
    handles: HashMap<String, Handle<Image>>,
}

fn main() {
    // 加载主配置
    let main_config = load_main_config();
    debug_print("var1",&main_config.title);
    let app_window = Some(Window {
        title: main_config.title.clone(),
        // 从配置文件读取分辨率
        resolution: (main_config.settings.resolution[0] as f32, main_config.settings.resolution[1] as f32).into(),
        ..default()
    });
    App::new()
    // 载入配置程序
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
        }))
        .insert_resource(main_config)  // 将配置作为资源插入
        // 设置背景清除颜色
        // 等效十六进制表示（深蓝紫色）
        // Color::srgb_u8(51, 51, 102) 
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.4)))
        .add_systems(Startup, (setup_camera, load_portraits, setup_ui))
        .add_systems(Update, (handle_input, update_dialogue, update_portrait))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // commands.spawn(Sprite::from_image(
    //     asset_server.load("background/main.png"),
    // ));
}

// 加载主配置文件
fn load_main_config() -> MainConfig {
    let yaml_str = fs::read_to_string("assets/main.yaml")
        .expect("找不到配置文件 assets/main.yaml");
    serde_yaml::from_str(&yaml_str)
        .expect("YAML解析失败，请检查格式")
}
// 从YAML加载对话数据，应用变量替换
fn load_dialogues(config: &MainConfig) -> Vec<Dialogue> {
    let yaml_str = fs::read_to_string("assets/dialogues.yaml")
        .expect("找不到对话文件 assets/dialogues.yaml");
    
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
    debug_print("var4",&processed_yaml);
    serde_yaml::from_str(&processed_yaml)
        .expect("YAML解析失败，请检查格式")
}
// 初始化游戏的状态
fn setup_camera(mut commands: Commands, config: Res<MainConfig>) {
    commands.spawn(Camera2d);
    commands.insert_resource(GameState {
        current_line: 0,
        dialogues: load_dialogues(&config),
        can_go_back: false, // 初始时不能返回
    });
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
        println!("{}",path_string);
        let handle = asset_server.load(&path_string);
        portrait_assets.handles.insert(character_name.clone(), handle);
    }
    println!("=== 所有立绘路径 ===");
    for (character_name, _) in &portrait_assets.handles {
        println!("角色: {}", character_name);
    }
    
    println!("==================");   
    // println!("{}",portrait_assets);
    commands.insert_resource(portrait_assets);
}
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    debug_print("var2",&asset_server);
    // 立绘容器
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
        Name::new("spritebox"),
        // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
        // Transform::from_xyz(0.0, 0.0, 0.0)
        // Sprite::sized(Vec2::new(75., 75.)),
        Sprite {
            image: asset_server.load("characters/protagonist/default.png"),
            custom_size: Some(Vec2 { x: 400.0, y: 600.0 }),
            ..default()
        },
        Visibility::Hidden,
    ));
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Name::new("textbox"),
        Text::new("文本框!"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size:28.0,
            ..default()
        },
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(150.0),
            padding: UiRect::all(Val::Px(10.0)),
            // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8).into();)
            ..default()
        },
        // 对话框背景颜色
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        // AnimatedText,
    ));
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Name::new("namebox"),
        Text::new("戴安娜"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size:28.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
        // TextColor(Color::srgba(0.6, 0.1, 0.1, 0.8)),
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // align_items: AlignItems::Center,
        // Set the style of the Node itself.
        // let background_color = 
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(210.0),
            left: Val::Px(50.0),
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
    game_state: Res<GameState>,
    mut query: Query<(&Name, &mut Text)> 
) {

    match game_state.dialogues.get(game_state.current_line) {
        Some(dialogue) => {
            for (name, mut text) in &mut query {
                // 比较方式1：转换为字符串切片
                if name.as_str() == "namebox" {
                    text.0 = format!("{}", dialogue.character);
                }
                if name.as_str() == "textbox" {
                    text.0 = format!("{}", dialogue.text);
                }
            }
            // println!("{}", format!("{}: {}", dialogue.character, dialogue.text)); 
        }
        None => {
            // println!("{:?}", dialogue);
            for (name, mut text) in &mut query {
                // 比较方式1：转换为字符串切片
                if name.as_str() == "namebox" {
                    text.0 = format!("{}", "NULL");
                }
                if name.as_str() == "textbox" {
                    text.0 = format!("{}", "感谢体验，按下ESC退出");
                }
            }
        }
    }
    
}

// 输入处理系统
fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
) {
    let click = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);
    
    if click {
        if game_state.current_line < game_state.dialogues.len() {
            game_state.current_line += 1;
            game_state.can_go_back = true; // 前进后可以返回
        }
    }
    let back_pressed = keys.just_pressed(KeyCode::Backspace) 
        || keys.just_pressed(KeyCode::ArrowLeft);

    if click {
        if game_state.current_line < game_state.dialogues.len() {
            game_state.can_go_back = true; // 前进后可以返回
            
        }
    }
    
    // 返回上一页
    if back_pressed && game_state.can_go_back && game_state.current_line > 0 {
        game_state.current_line -= 1;
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
    mut query: Query<(&mut Sprite,&mut Name,&mut Visibility)>,
) {
    // 先保存查询结果到变量
    // 遍历所有实体，检查名称
    for (mut texture_handle, mut name, mut visibility) in query.iter_mut() {
        if name.as_str() == "spritebox" {
            // 检查当前是否有对话
            if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
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