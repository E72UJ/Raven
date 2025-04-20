use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

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
    narrator: Handle<Image>,
    alice: Handle<Image>,
    bob: Handle<Image>,
}

fn main() {
    let app_window = Some(Window {
        title: "Raven 引擎 \"V 0.1\"".into(),
        ..default()
      });
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: app_window,
            ..default()
          }))
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.4)))
        .add_systems(Startup, (setup_camera, load_portraits, setup_ui))
        .add_systems(Update, (handle_input, update_dialogue, update_portrait))

        .run();
}

// 从YAML加载对话数据
fn load_dialogues() -> Vec<Dialogue> {
    let yaml_str = fs::read_to_string("assets/dialogues.yaml")
        .expect("找不到对话文件 assets/dialogues.yaml");
    serde_yaml::from_str(&yaml_str)
        .expect("YAML解析失败，请检查格式")
}

// 初始化游戏状态
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(GameState {
        current_line: 0,
        dialogues: load_dialogues(),
        can_go_back: false, // 初始时不能返回
    });
}

// 加载立绘资源
fn load_portraits(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PortraitAssets {
        narrator: asset_server.load("portraits/narrator.png"),
        alice: asset_server.load("portraits/alice.png"),
        bob: asset_server.load("portraits/bob.png"),
    });
}

// 创建UI界面
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    mut text_query: Query<&mut Text>,
) {
    let mut text = text_query.single_mut();
    
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
        match dialogue.portrait.as_str() {
            "narrator" => image.texture = portraits.narrator.clone(),
            "alice" => image.texture = portraits.alice.clone(),
            "bob" => image.texture = portraits.bob.clone(),
            _ => *visibility = Visibility::Hidden,
        }
    } else {
        *visibility = Visibility::Hidden;
    }
}