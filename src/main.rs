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

// 打字机效果组件
#[derive(Component)]
struct TypewriterEffect {
    full_text: String,
    current_length: usize,  // 字符数量而非字节索引
    timer: Timer,
    is_complete: bool,
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
        .add_systems(Update, (handle_input, update_dialogue, update_portrait, typewriter_system))
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
        // 添加打字机效果组件
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 28.0,
                    color: Color::WHITE,
                },
            ),
            TypewriterEffect {
                full_text: "".to_string(),
                current_length: 0,
                timer: Timer::from_seconds(0.03, TimerMode::Repeating),
                is_complete: false,
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
    mut typewriter_query: Query<&mut TypewriterEffect>,
) {
    let mut typewriter = typewriter_query.single_mut();
    
    // 如果打字机效果尚未完成，快进到完整文本
    if !typewriter.is_complete && (keys.just_pressed(KeyCode::Space) 
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left)) {
        typewriter.current_length = typewriter.full_text.chars().count();
        typewriter.is_complete = true;
        return;
    }
    
    // 前进 - 只有当打字机效果完成后才能前进到下一行
    let forward_pressed = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse.just_pressed(MouseButton::Left);

    // 返回
    let back_pressed = keys.just_pressed(KeyCode::Backspace) 
        || keys.just_pressed(KeyCode::ArrowLeft);

    if forward_pressed && typewriter.is_complete {
        if game_state.current_line < game_state.dialogues.len() {
            game_state.current_line += 1;
            game_state.can_go_back = true; // 前进后可以返回
            // 重置打字机状态
            typewriter.current_length = 0;
            typewriter.is_complete = false;
        }
    }
    
    // 返回上一页
    if back_pressed && game_state.can_go_back && game_state.current_line > 0 {
        game_state.current_line -= 1;
        if game_state.current_line == 0 {
            game_state.can_go_back = false; // 回到开始时不能再返回
        }
        // 重置打字机状态
        typewriter.current_length = 0;
        typewriter.is_complete = false;
    }
    
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

// 更新对话文本
fn update_dialogue(
    game_state: Res<GameState>,
    mut typewriter_query: Query<&mut TypewriterEffect>,
) {
    let mut typewriter = typewriter_query.single_mut();
    
    match game_state.dialogues.get(game_state.current_line) {
        Some(dialogue) => {
            let new_text = format!("{}: {}", dialogue.character, dialogue.text);
            // 只有当全文变化时才重置打字机效果
            if typewriter.full_text != new_text {
                typewriter.full_text = new_text;
                typewriter.current_length = 0;
                typewriter.is_complete = false;
            }
        }
        None => {
            typewriter.full_text = "感谢体验！按ESC退出".to_string();
            typewriter.current_length = 0;
            typewriter.is_complete = false;
            if game_state.current_line >= game_state.dialogues.len() {
                std::process::exit(0);
            }
        }
    }
}

// 打字机系统
fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut TypewriterEffect, &mut Text)>,
) {
    for (mut effect, mut text) in query.iter_mut() {
        // 如果已经完成，无需继续
        if effect.is_complete {
            continue;
        }
        
        // 更新计时器
        effect.timer.tick(time.delta());
        
        // 当计时器完成一个周期时
        if effect.timer.just_finished() {
            // 如果还有字符要显示
            if effect.current_length < effect.full_text.chars().count() {
                // 取前N个字符
                let current_text: String = effect.full_text.chars().take(effect.current_length + 1).collect();
                text.sections[0].value = current_text;
                
                // 移动到下一个字符
                effect.current_length += 1;
            } else {
                // 全部字符已显示
                effect.is_complete = true;
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