use bevy::prelude::*;
use serde::Deserialize;
use std::{fs, path};
use std::collections::HashMap;
use std::path::PathBuf;

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
#[derive(Resource)]
struct GameState {
    current_line: usize,
    dialogues: Vec<Dialogue>,
    can_go_back: bool, // 添加标志位判断是否可以返回
}

// 立绘组件
#[derive(Component)]
struct Portrait;

// 立绘资源句柄
#[derive(Resource)]
struct PortraitAssets {
    handles: HashMap<String, Handle<Image>>,
}

fn main() {
    // 加载主配置
    let main_config = load_main_config();
    
    let app_window = Some(Window {
        title: main_config.title.clone(),
        // 从配置文件读取分辨率
        resolution: (main_config.settings.resolution[0] as f32, main_config.settings.resolution[1] as f32).into(),
        ..default()
    });
    
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
        }))
        .insert_resource(main_config)  // 将配置作为资源插入
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.4)))
        .add_systems(Startup, (setup_camera, load_portraits, setup_ui))
        .add_systems(Update, (handle_input, update_dialogue, update_portrait))
        .run();
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
    
    serde_yaml::from_str(&processed_yaml)
        .expect("YAML解析失败，请检查格式")
}

// 初始化游戏状态
fn setup_camera(mut commands: Commands, config: Res<MainConfig>) {
    commands.spawn(Camera2dBundle::default());
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
    commands.insert_resource(portrait_assets);
}

// 创建UI界面
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    // 立绘容器
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            z_index: ZIndex::Global(1),
            ..default()
        },
        Portrait,
    )).with_children(|parent| {
        parent.spawn(ImageBundle {
            image: UiImage::default(),
            style: Style {
                width: Val::Px(400.0),
                height: Val::Px(600.0),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        });
    });

    // 对话框
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            height: Val::Px(150.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        background_color: Color::rgba(0.1, 0.1, 0.1, 0.8).into(),
        z_index: ZIndex::Global(2),
        ..default()
    }).with_children(|parent| {
        // 添加文本
        parent.spawn(TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 28.0,
                color: Color::WHITE,
            },
        ));
    });

    // 点击区域
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::NONE.into(),
        z_index: ZIndex::Global(3),
        ..default()
    });
}

// 输入处理系统
fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
) {
    // 前进
    let forward_pressed = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);

    // 返回
    let back_pressed = keys.just_pressed(KeyCode::Backspace) 
        || keys.just_pressed(KeyCode::ArrowLeft);

    if forward_pressed {
        if game_state.current_line < game_state.dialogues.len() {
            game_state.current_line += 1;
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
    
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

// 更新对话文本
fn update_dialogue(
    game_state: Res<GameState>,
    mut query: Query<&mut Text>,
) {
    let mut text = query.single_mut();
    
    match game_state.dialogues.get(game_state.current_line) {
        Some(dialogue) => {
            text.sections[0].value = format!("{}: {}", dialogue.character, dialogue.text);
        }
        None => {
            text.sections[0].value = "感谢体验！按ESC退出".to_string();
            if game_state.current_line >= game_state.dialogues.len() {
                std::process::exit(0);
            }
        }
    }
}

// 更新立绘显示
fn update_portrait(
    game_state: Res<GameState>,
    portraits: Res<PortraitAssets>,
    mut query: Query<(&mut UiImage, &mut Visibility)>,
) {
    let (mut image, mut visibility) = query.single_mut();

    if let Some(dialogue) = game_state.dialogues.get(game_state.current_line) {
        *visibility = Visibility::Visible;
        // println!("当前对话使用的立绘: {}", dialogue.portrait);

        // 从角色映射中获取立绘
        if let Some(handle) = portraits.handles.get(&dialogue.portrait) {
            image.texture = handle.clone();
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}