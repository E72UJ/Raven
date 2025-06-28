use bevy::prelude::*;

// use bevy_svg::prelude::*;
use serde::Deserialize;
use serde_yaml::with;
use std::collections::HashMap;
use std::env;
use std::fs;
use bevy::audio::{ AudioPlugin, PlaybackSettings};
use std::path::PathBuf;
use bevy::{
    prelude::*,
    ui::FocusPolicy, // 添加这行
    // ... 其他导入
};
// 正确的导入方式
use bevy::{
    color::palettes::basic::*, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    winit::WinitSettings,
};
use bevy_flash::{FlashPlugin, assets::FlashAnimationSwfData, bundle::FlashAnimation};
use bevy::{audio::Volume, math::ops, prelude::*};
pub const FPS_OVERLAY_Z_INDEX: i32 = i32::MAX - 32;


// 按钮组颜色表格

const NORMAL_BUTTON: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);



#[derive(Resource, Default)]
struct ChoiceState {
    current_choices: Vec<Choice>,
    buttons_created: bool,
    last_dialogue_line: usize,
}

#[derive(Component)]
struct ClickArea;

#[derive(Component)]
struct ChoiceButton1;

#[derive(Component)]
struct ChoiceButton2;

#[derive(Component)]
struct CurrentText;

#[derive(Component)]
struct DialogueBox;
// 按钮组颜色表格结束
// 主配置结构体

#[derive(Resource)]
struct DialogueChanged(pub bool);

impl Default for DialogueChanged {
    fn default() -> Self {
        Self(false)
    }
}
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

#[derive(Debug, Deserialize)]
struct Choice {
    text: String,
    goto: String,
}
#[derive(Component)]
struct ButtonContainer;
// 添加这些组件定义
#[derive(Component)]
struct DynamicButton;

#[derive(Component)]
struct ClickHandler(String);

// 对话数据结构（支持YAML反序列化）
#[derive(Debug, Deserialize)]
struct Dialogue {
    character: String,
    text: String,
    portrait: String,
    #[serde(default)] // 如果YAML中没有这个字段，使用默认值false
    branch_visible: bool, // 新增字段
    choices: Option<Vec<Choice>>, // 动态的分支选项
}
// 游戏状态资源
#[derive(Debug, Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
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
fn main() {
    println!("{:?}", "mac 主程序启动！");
    // 加载主配置
    let main_config = load_main_config();
    let app_window = Some(Window {
        title: main_config.title.clone(),
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
        .add_systems(Startup, (setup_camera, load_portraits, setup_ui,load_audio_resources))
        // 更新系统
        .add_systems(Update, (handle_input, update_dialogue, update_portrait,flash_animation,keyboard_system, create_dynamic_buttons.run_if(resource_changed::<GameState>),button_interaction_system,handle_choice_buttons))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>,config: Res<MainConfig>,
    game_state: Res<GameState>,) {
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

    commands.insert_resource(GameState {
        current_line: 0,
        dialogues: load_dialogues(&config),
        can_go_back: false, // 初始时不能返回
        in_branch_selection: false, // 初始化为false
        
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
    // 点击区域容器
    let mut click_area_entity = commands
        .spawn((
            Name::new("click_area"),
            // Button, // 添加这行
            ClickArea,
            Node {
                width: Val::Px(1200.0),     // 固定宽度800像素
                height: Val::Px(660.0),    // 固定高度600像素
                bottom: Val::Px(50.0),
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
        // .id();
// 分支创建============
commands.spawn((
        Name::new("choice_container"),
        ButtonContainer,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(250.0), // 在对话框上方
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
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(123),
        // BackgroundColor(Color::srgba(0.4, 0.4, 0.1, 1.0)),
        Portrait,
    ));
    commands.spawn((
        Name::new("svgload"),
        FlashAnimation {
            // name:"a1",
            swf: asset_server.load("swf/66.swf"),
        },
        Transform::from_translation(Vec3::new(-400.0, 240.0, 0.0)).with_scale(Vec3::splat(2.0)),
    ));
    commands.spawn((
        Name::new("spritebox"),
        // Sprite::from_color(Color::srgba(0.4, 0.4, 0.1, 1.0), Vec2::new(400.0, 600.0)),
        Transform::from_xyz(1.0, 2.0, 0.0),
        // Sprite::sized(Vec2::new(75., 75.)),
        Sprite {
            image: asset_server.load("characters/protagonist/default.png"),
            custom_size: Some(Vec2 { x: 400.0, y: 600.0 }),
            ..default()
        },
        Visibility::Hidden,
    ));
    // 按钮组导航
    commands
    .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        bottom: Val::Px(-10.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                    GlobalZIndex(3),
                ))
                .with_children(|parent| {
                    // 创建所有导航按钮
                    let nav_items = vec!["主菜单", "保存", "读取", "设置", "历史", "跳过", "自动"];
                    
                    for item in nav_items {
                        create_nav_button(parent, &asset_server, item);
                    }
                        // create_nav_button2(parent, &asset_server, "分支1");
                        // create_nav_button2(parent, &asset_server, "分支2");
                });
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
                padding: UiRect::all(Val::Px(30.0)),
                // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8).into();),
                ..default()
            },
            
            // 对话框背景颜色
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            DialogueBox, // 添加这个组件
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
                CurrentText
            ));

        });
        
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Name::new("namebox"),
        Text::new("戴安娜"),
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
        // 对话框背景颜色
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        GlobalZIndex(2),
        // AnimatedText,
    ));
    // 
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            bottom: Val::Px(50.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        // BackgroundColor(Color::srgba(0.4, 0.4, 0.1, 1.0)),
        GlobalZIndex(2),
        // Portrait,
    ));
}

// 更新对话文本
fn update_dialogue(game_state: Res<GameState>, mut query: Query<(&Name, &mut Text)>) {
    match game_state.dialogues.get(game_state.current_line) {
        Some(dialogue) => {
            for (name, mut text) in &mut query {
                // 比较方式1：转换为字符串切片
                if name.as_str() == "namebox" {
                    text.0 = dialogue.character.to_string();
                }
                if name.as_str() == "textbox" {
                    text.0 = dialogue.text.to_string();
                }
            }
            // println!("{}", format!("{}: {}", dialogue.character, dialogue.text));
        }
        None => {
            // println!("{:?}", dialogue);
            for (name, mut text) in &mut query {
                // 比较方式1：转换为字符串切片
                if name.as_str() == "namebox" {
                    text.0 = "NULL".to_string();
                }
                if name.as_str() == "textbox" {
                    text.0 = "感谢体验，按下ESC退出".to_string();
                }
            }
        }
    }
}
// 输入处理系统
fn handle_input(
    mut interaction_query: Query<(&Interaction, &Name), (Changed<Interaction>, With<Node>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    mut commands: Commands,
) {
    // ESC键始终可用
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // 如果在分支选择状态，只禁用前进操作（空格键、回车键、点击区域）
    if game_state.in_branch_selection {
        // 在分支选择状态下，不处理前进相关的输入
        return;
    }

    // 检查键盘输入 - 只在非分支选择状态下处理前进操作
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        // play_sound(&click_sound.0, commands);
        if game_state.current_line < game_state.dialogues.len() {
            game_state.current_line += 1;
            game_state.can_go_back = true;
        }
        return;
    }
    
    // 检查鼠标点击事件 - 只在非分支选择状态下处理
    for (interaction, name) in &interaction_query {
        if *interaction == Interaction::Pressed && name.as_str() == "click_area" {
            // play_sound(&click_sound.0, commands);
            if game_state.current_line < game_state.dialogues.len() {
                game_state.current_line += 1;
                game_state.can_go_back = true;
            }
            println!("点击了透明区域");
            break;
        }
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
// 预加载系统
// fn preload_sounds(asset_server: Res<AssetServer>) {
//     asset_server.load::<AudioSource>("button.ogg");
// }
fn create_nav_button(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    label: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::all(Val::Px(5.0)),
            // BackgroundColor(NORMAL_BUTTON),
            Name::new(label.to_string()),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
}


fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Name,
        ),
        (With<Button>),
    >,
) {
      println!("button_system 被调用了，查询到 {} 个实体", interaction_query.iter().count());
    for (interaction, mut color, mut border_color, name) in &mut interaction_query {
         println!("处理按钮: {}, 状态: {:?}", name.as_str(), interaction);
        if name.as_str() == "click_area" {
            // 点击区域始终保持完全透明
            *color = Color::NONE.into();
            border_color.0 = Color::NONE;
            
            // 如果需要在点击时执行某些操作，可以在这里添加
            if *interaction == Interaction::Pressed {
                println!("点击了透明区域");
                // 在这里添加您需要的点击逻辑
            }
        } else {
            // 其他按钮的正常处理
            match *interaction {
                Interaction::Pressed => {
                    *color = PRESSED_BUTTON.into();
                    border_color.0 = Color::srgba(0.1, 0.1, 0.1, 0.8);
                    println!("按下了: {}", name.as_str());
                }
                Interaction::Hovered => {
                    println!("设置悬停样式: {:?} -> {:?}", color.0, HOVERED_BUTTON);
                    *color = HOVERED_BUTTON.into();
                    border_color.0 = Color::WHITE;
                    println!("执行了一个数据");
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                    border_color.0 = Color::BLACK;
                }
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


// 分支
fn control_branch_visibility(
    mut dynamic_button_query: Query<&mut Visibility, With<DynamicButton>>,
    game_state: Res<GameState>,
) {
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        if dialogue.choices.is_some() {
            // 当前对话有选择项，显示所有动态按钮
            for mut visibility in dynamic_button_query.iter_mut() {
                *visibility = Visibility::Visible;
            }
        } else {
            // 当前对话没有选择项，隐藏所有动态按钮
            for mut visibility in dynamic_button_query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
fn create_dynamic_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
    existing_buttons: Query<Entity, With<DynamicButton>>,
    button_container: Query<Entity, With<ButtonContainer>>,
     mut click_area_query: Query<&mut Visibility, With<ClickArea>>,
) {
    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        println!("当前对话行: {}, 角色: {}", game_state.current_line, dialogue.character);
        
        if let Some(choices) = &dialogue.choices {
            println!("发现 {} 个选择分支", choices.len());
                        // 隐藏点击区域来禁用交互
            if let Ok(mut visibility) = click_area_query.get_single_mut() {
                *visibility = Visibility::Hidden;
            }
            // 先清除现有按钮
            for entity in existing_buttons.iter() {
                commands.entity(entity).despawn_recursive();
            }
            
            if let Ok(container) = button_container.get_single() {
                for (index, choice) in choices.iter().enumerate() {
                    commands.entity(container).with_children(|parent| {
                        parent.spawn((
                            Button,
                            DynamicButton,
                            ClickHandler(choice.goto.to_string()),
                            Interaction::default(),
                            Name::new(format!("choice_{}", index)),
                            // 关键修复：确保交互组件和样式正确设置
                            Node {
                                // 分支按钮样式
                                position_type: PositionType::Relative,   // 绝对定位
                                bottom: Val::Px(100.0),
                                    // 绝对定位的位置
                                top: Val::Px(-220.0),
                                left: Val::Px(410.0),
                                width: Val::Px(400.0),
                                height: Val::Px(40.0),
                                border: UiRect::all(Val::Px(2.0)), // 增加边框宽度便于看到效果
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                // margin: UiRect::all(Val::Px(5.0)),
                                padding: UiRect {
                                    left: Val::Px(2.0),
                                    right: Val::Px(2.0),
                                    top: Val::Px(5.0),
                                    bottom: Val::Px(5.0),
                                },
                                ..default()
                            },
                            // 使用你定义的常量设置初始状态
                            BackgroundColor(NORMAL_BUTTON),
                            BorderColor(Color::BLACK),
                            BorderRadius::all(Val::Px(5.0)),
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
        } else {
            println!("当前对话没有选择分支，清除所有按钮");
            if let Ok(mut visibility) = click_area_query.get_single_mut() {
                *visibility = Visibility::Visible;
            }
            for entity in existing_buttons.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
fn handle_choice_buttons(
    mut interaction_query: Query<(&Interaction, &ClickHandler), (Changed<Interaction>, With<DynamicButton>)>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    mut commands: Commands,
) {
    for (interaction, click_handler) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // play_sound(&click_sound.0, commands);
            
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
// 添加新系统
fn initialize_new_buttons(
    mut button_query: Query<
        (&mut BackgroundColor, &mut BorderColor, &Name),
        (With<Button>, Added<Button>) // 关键：Added<Button>
    >,
) {
    for (mut color, mut border_color, name) in &mut button_query {
        if name.as_str() != "click_area" {
            *color = NORMAL_BUTTON.into();
            border_color.0 = Color::BLACK;
        }
    }
}
// 原有的交互系统保持不变
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
// 创建分支按钮
