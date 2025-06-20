use bevy::prelude::*;
use bevy::text::cosmic_text::ttf_parser::Style;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use bevy::audio::{ AudioPlugin, PlaybackSettings};
use std::path::PathBuf;
use bevy::{
    color::palettes::basic::*, ecs::relationship::RelatedSpawnerCommands, prelude::*,
    winit::WinitSettings,
};
use bevy_flash::{FlashPlugin, assets::FlashAnimationSwfData, bundle::FlashAnimation};
use bevy::{audio::Volume, math::ops, prelude::*};

pub const FPS_OVERLAY_Z_INDEX: i32 = i32::MAX - 32;

// 游戏状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
enum GameState {
    #[default]
    SplashScreen,  // 开屏动画状态
    MainGame,      // 主游戏状态
}

// 开屏动画组件标识
#[derive(Component)]
struct SplashScreenEntity;

// 开屏动画计时器
#[derive(Resource)]
struct SplashTimer(Timer);

// 按钮组颜色表格
const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// 位置常量
const left_box: f32 = 50.0;

// 背景组件标识
#[derive(Component)]
struct Background;

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
    click_sound: String,
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
#[derive(Debug, Resource)]
struct LabelMap(HashMap<String, usize>);

// 对话数据结构
#[derive(Debug, Deserialize)]
struct Dialogue {
    character: String,
    text: String,
    portrait: String,
    background: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    jump: Option<String>,
}

// 对话游戏状态资源
#[derive(Debug, Resource)]
struct DialogueState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool,
    jump_label: Option<String>,
}

// 立绘组件
#[derive(Component)]
struct Portrait;

// 定义音频句柄资源
#[derive(Resource)]
struct ClickSound(Handle<AudioSource>);

#[derive(Resource)]
struct BackClickSound(Handle<AudioSource>);

#[derive(Debug, Resource)]
struct PortraitAssets {
    handles: HashMap<String, Handle<Image>>,
}

// 音频控制
#[derive(Component)]
struct MyMusic;

fn main() {
    println!("{:?}", "mac 主程序启动！");
    
    let main_config = load_main_config();
    let app_window = Some(Window {
        title: main_config.title.clone(),
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
        ..default()
    });

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
        }))
        .add_plugins(FlashPlugin)
        .insert_resource(main_config)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.2, 0.4)))
        .init_state::<GameState>()
        .insert_resource(SplashTimer(Timer::from_seconds(3.0, TimerMode::Once))) // 3秒开屏动画
        
        // 开屏动画系统
        .add_systems(OnEnter(GameState::SplashScreen), setup_splash_screen)
        .add_systems(Update, (
            handle_splash_screen,
            flash_animation,
        ).run_if(in_state(GameState::SplashScreen)))
        .add_systems(OnExit(GameState::SplashScreen), cleanup_splash_screen)
        
        // 主游戏系统
        .add_systems(OnEnter(GameState::MainGame), (
            setup_camera,
            load_portraits,
            setup_ui,
            load_audio_resources,
            load_backgrounds
        ))
        .add_systems(Update, (
            handle_input,
            update_dialogue,
            update_portrait,
            apply_jump,
            update_background
        ).run_if(in_state(GameState::MainGame)))
        
        .run();
}

// 设置开屏动画
fn setup_splash_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    // 摄像机
    commands.spawn(Camera2d);
    
    // Flash 开屏动画
    commands.spawn((
        Name::new("splash_animation"),
        FlashAnimation {
            swf: asset_server.load("swf/345_c7.swf")
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_scale(Vec3::splat(2.0)),
        SplashScreenEntity,
    ));
    
    // 可选：添加背景或其他装饰元素
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.2),
            custom_size: Some(Vec2::new(1200.0, 800.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        SplashScreenEntity,
    ));
    
    // 可选：添加"Loading..."文本
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(100.0),
            left: Val::Percent(50.0),
            ..default()
        },
        SplashScreenEntity,
    ));
}

// 处理开屏动画逻辑
fn handle_splash_screen(
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: ResMut<SplashTimer>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    // 更新计时器
    timer.0.tick(time.delta());
    
    // 检查是否应该跳过开屏动画（用户输入或计时器到期）
    let should_skip = timer.0.finished() 
        || keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Escape)
        || mouse.just_pressed(MouseButton::Left);
    
    if should_skip {
        println!("开屏动画结束，进入主游戏");
        next_state.set(GameState::MainGame);
    }
}

// 清理开屏动画
fn cleanup_splash_screen(
    mut commands: Commands,
    query: Query<Entity, With<SplashScreenEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    println!("开屏动画资源已清理");
}

// Flash动画控制（保持原有逻辑，但只在开屏状态运行）
fn flash_animation(
    mut flashes: ResMut<Assets<FlashAnimationSwfData>>,
    mut flash_swf_data_events: EventReader<AssetEvent<FlashAnimationSwfData>>,
) {
    for event in flash_swf_data_events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            if let Some(flash) = flashes.get_mut(*id) {
                flash.player.set_on_completion(Box::new(|player| {
                    let _ = player.set_play_animation("default", false);
                }));
                
                let _ = flash.player.set_play_animation("default", true);
            }
        }
    }
}

// 加载主配置文件（保持不变）
fn load_main_config() -> MainConfig {
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let yaml_path3 = exe_dir.join("assets/main.yaml");
    println!("相对的配置路径: {:?}", yaml_path3);
    let yaml_str = fs::read_to_string(yaml_path3).expect("找不到配置文件 assets/main.yaml");
    serde_yaml::from_str(&yaml_str).expect("YAML解析失败，请检查格式")
}

// 从YAML加载对话数据（保持不变）
fn load_dialogues(config: &MainConfig) -> Vec<Dialogue> {
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let yaml_path2 = exe_dir.join("assets/dialogues.yaml");
    let yaml_str = fs::read_to_string(yaml_path2).expect("找不到对话文件 assets/dialogues.yaml");

    let mut processed_yaml = yaml_str.clone();

    for (var_name, var_value) in &config.global_variables {
        processed_yaml = processed_yaml.replace(&format!("${}", var_name), var_value);
    }

    processed_yaml = processed_yaml.replace("$title", &config.title);

    for (bg_name, bg_path) in &config.assets.backgrounds {
        processed_yaml = processed_yaml.replace(&format!("$backgrounds.{}", bg_name), bg_path);
    }

    for (bgm_name, bgm_path) in &config.assets.audio.bgm {
        processed_yaml = processed_yaml.replace(&format!("$audio.bgm.{}", bgm_name), bgm_path);
    }

    for (sfx_name, sfx_path) in &config.assets.audio.sfx {
        processed_yaml = processed_yaml.replace(&format!("$audio.sfx.{}", sfx_name), sfx_path);
    }

    for (char_name, char_path) in &config.assets.characters {
        processed_yaml = processed_yaml.replace(&format!("$characters.{}", char_name), char_path);
    }

    serde_yaml::from_str(&processed_yaml).expect("YAML解析失败，请检查格式")
}

// 初始化主游戏摄像机和对话系统
fn setup_camera(mut commands: Commands, config: Res<MainConfig>) {
    // 注意：摄像机已在开屏动画中创建，这里可以重新配置或保持原有
    let dialogues: Vec<Dialogue> = load_dialogues(&config);
    
    let mut label_map = HashMap::new();
    for (index, dialogue) in dialogues.iter().enumerate() {
        if let Some(label) = dialogue.label.as_ref() {
            label_map.insert(label.clone(), index);
        }
    }
    
    commands.insert_resource(DialogueState {
        current_line: 0,
        dialogues: load_dialogues(&config),
        can_go_back: false,
        jump_label: None
    });
    
    commands.insert_resource(LabelMap(label_map));
}

// 其余函数保持不变，但需要将 GameState 改为 DialogueState
fn load_portraits(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    let mut portrait_assets = PortraitAssets {
        handles: HashMap::new(),
    };

    for (character_name, character_path) in &config.assets.characters {
        let character_path = if character_path.starts_with("assets/") {
            character_path.trim_start_matches("assets/").to_string()
        } else {
            character_path.clone()
        };

        let path_string = format!("{}/default.png", character_path.replace('\\', "/"));
        println!("{}", path_string);
        let handle = asset_server.load(&path_string);
        portrait_assets.handles.insert(character_name.clone(), handle);
    }
    
    println!("=== 所有立绘路径 ===");
    for character_name in portrait_assets.handles.keys() {
        println!("角色: {}", character_name);
    }
    println!("==================");
    
    commands.insert_resource(portrait_assets);
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    // UI设置代码保持不变，但移除了FlashAnimation相关代码
    commands.spawn((
        Name::new("sidebox"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            left: Val::Px(0.0),
            top: Val::Px(80.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(10000),
        ZIndex(1200),
    )).with_children(|parent| {
        parent.spawn((
            Name::new("textbox"),
            ImageNode::new(asset_server.load("characters/protagonist/02.png")),
            Visibility::Hidden,
            Transform::from_translation(Vec3::new(1450.0, -750.0, 0.0)).with_scale(Vec3::new(0.5, 0.5, 0.0)),
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
        Portrait,
    ));

    commands.spawn((
        Name::new("spritebox"),
        Transform::from_xyz(1.0, 1.0, 0.0),
        Sprite {
            image: asset_server.load("characters/protagonist/default.png"),
            custom_size: Some(Vec2 { x: 400.0, y: 600.0 }),
            ..default()
        },
        Visibility::Hidden,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(170.0),
                padding: UiRect {
                    left: Val::Px(30.0),
                    right: Val::Px(30.0),
                    top: Val::Px(30.0),
                    bottom: Val::Px(30.0),
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("textbox"),
                Text::new("文本框!"),
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
            ));
        });

    commands.spawn((
        Name::new("namebox"),
        Text::new("戴安娜"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 28.0,
            line_height: bevy::text::LineHeight::Px(50.),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(230.0),
            left: Val::Px(left_box),
            right: Val::Px(50.0),
            height: Val::Px(50.0),
            width: Val::Px(220.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        GlobalZIndex(2),
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        GlobalZIndex(5),
    ));
}

// 更新对话文本（将GameState改为DialogueState）
fn update_dialogue(
    mut dialogue_state: ResMut<DialogueState>,
    label_map: Res<LabelMap>,
    mut query: Query<(&Name, &mut Text)>,
) {
    let current_dialogue = if let Some(dialogue) = dialogue_state.dialogues.get(dialogue_state.current_line) {
        dialogue
    } else {
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
    
    for (name, mut text) in &mut query {
        if name.as_str() == "namebox" {
            text.0 = current_dialogue.character.to_string();
        }
        if name.as_str() == "textbox" {
            text.0 = current_dialogue.text.to_string();
        }
    }
    
    if let Some(jump_label) = &current_dialogue.jump {
        if let Some(&new_line) = label_map.0.get(jump_label) {
            println!(
                "显示行 {}: 角色='{}', 标签={:?}, 跳转={:?}",
                dialogue_state.current_line,
                current_dialogue.character,
                current_dialogue.label,
                current_dialogue.jump
            );
        } else {
            println!("错误: 找不到标签 '{}' 的跳转目标", jump_label);
        }
    }
}

// 输入处理系统（将GameState改为DialogueState）
fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    click_sound: Res<ClickSound>,
    back_sound: Res<BackClickSound>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
    mut commands: Commands,
    mut dialogue_state: ResMut<DialogueState>,
    label_map: Res<LabelMap>,
) {
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Digit0 => dialogue_state.current_line = 0,
            KeyCode::Digit1 => dialogue_state.current_line = 1,
            KeyCode::Digit2 => dialogue_state.current_line = 2,
            _ => {}
        }
    };

    let click = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);

    if click && dialogue_state.current_line < dialogue_state.dialogues.len() {
        let current_dialogue = &dialogue_state.dialogues[dialogue_state.current_line];
        
        if let Some(jump_label) = &current_dialogue.jump {
            dialogue_state.jump_label = Some(jump_label.clone());
        } else {
            dialogue_state.current_line += 1;
        }
        
        dialogue_state.can_go_back = true;
        play_sound(&click_sound.0, commands);
    }

    let back_pressed =
        keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::ArrowLeft);

    if click && dialogue_state.current_line < dialogue_state.dialogues.len() {
        dialogue_state.can_go_back = true;
    }

    if back_pressed && dialogue_state.can_go_back && dialogue_state.current_line > 0 {
        dialogue_state.current_line -= 1;
        if dialogue_state.current_line == 0 {
            dialogue_state.can_go_back = false;
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn update_portrait(
    dialogue_state: Res<DialogueState>,
    portraits: Res<PortraitAssets>,
    mut query: Query<(&mut Sprite, &mut Name, &mut Visibility)>,
) {
    for (mut texture_handle, name, mut visibility) in query.iter_mut() {
        if name.as_str() == "spritebox" {
            if let Some(dialogue) = dialogue_state.dialogues.get(dialogue_state.current_line) {
                match portraits.handles.get(&dialogue.portrait) {
                    Some(handle) => {
                        texture_handle.image = handle.clone();
                        *visibility = Visibility::Visible;
                    }
                    None => {
                        *visibility = Visibility::Hidden;
                        eprintln!("找不到立绘资源: {}", dialogue.portrait);
                    }
                }
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn load_audio_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    let click_sound_handle: Handle<AudioSource> = asset_server.load(&config.assets.audio.click_sound);
    let backclick_sound_handle: Handle<AudioSource> = asset_server.load(&config.assets.audio.click_sound);
    
    commands.insert_resource(ClickSound(click_sound_handle));
    commands.insert_resource(BackClickSound(backclick_sound_handle));
}

fn play_sound(audio_handle: &Handle<AudioSource>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer::new(audio_handle.clone()),
        PlaybackSettings::ONCE,
    ));
}

fn apply_jump(
    label_map: Res<LabelMap>,
    mut dialogue_state: ResMut<DialogueState>,
) {
    if let Some(jump_label) = dialogue_state.jump_label.take() {
        if let Some(&target_line) = label_map.0.get(&jump_label) {
            println!("执行跳转: {} → {}", dialogue_state.current_line, target_line);
            dialogue_state.current_line = target_line;
            dialogue_state.can_go_back = true;
        } else {
            eprintln!("错误: 找不到标签 '{}' 的跳转目标", jump_label);
        }
    }
}

fn load_backgrounds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<MainConfig>,
) {
    for (bg_name, bg_path) in &config.assets.backgrounds {
        commands.spawn((
            Name::new(format!("background_{}", bg_name)),
            Background,
            Sprite {
                image: asset_server.load(bg_path),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -10.0),
            Visibility::Hidden,
        ));
    }
    
    println!("=== 已加载背景 ===");
    for bg_name in config.assets.backgrounds.keys() {
        println!("背景: {}", bg_name);
    }
    println!("==================");
}

fn update_background(
    dialogue_state: Res<DialogueState>,
    mut query: Query<(&Name, &mut Visibility), With<Background>>,
) {
    if let Some(dialogue) = dialogue_state.dialogues.get(dialogue_state.current_line) {
        for (_, mut visibility) in query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        
        if let Some(bg_name) = &dialogue.background {
            for (name, mut visibility) in query.iter_mut() {
                if name.as_str() == format!("background_{}", bg_name) {
                    *visibility = Visibility::Visible;
                }
            }
        }
    }
}