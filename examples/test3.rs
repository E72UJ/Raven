use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// 脚本节点枚举
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ScriptNode {
    #[serde(rename = "dialogue")]
    Dialogue {
        character: String,
        text: String,
        portrait: String,
        background: Option<String>,
    },
    #[serde(rename = "transition")]
    Transition {
        transition: String,
        duration: f32,
        from_bg: Option<String>,
        to_bg: Option<String>,
    },
}

// 转场组件
#[derive(Component)]
pub struct TransitionEffect {
    pub transition_type: TransitionType,
    pub duration: f32,
    pub elapsed: f32,
    pub from_bg: Option<String>,
    pub to_bg: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TransitionType {
    Fade,
    SlideLeft,
    SlideRight,
    Crossfade,
    Dissolve,
}

impl From<&str> for TransitionType {
    fn from(s: &str) -> Self {
        match s {
            "fade" => TransitionType::Fade,
            "slide_left" => TransitionType::SlideLeft,
            "slide_right" => TransitionType::SlideRight,
            "crossfade" => TransitionType::Crossfade,
            "dissolve" => TransitionType::Dissolve,
            _ => TransitionType::Fade,
        }
    }
}

// 背景图片组件
#[derive(Component)]
pub struct BackgroundImage {
    pub name: String,
}

// UI组件标记
#[derive(Component)]
pub struct DialogueBox;

#[derive(Component)]
pub struct CharacterName;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct TransitionOverlay;

// 脚本执行状态
#[derive(Resource)]
pub struct ScriptState {
    pub nodes: Vec<ScriptNode>,
    pub current_index: usize,
    pub is_transitioning: bool,
}

// 初始化系统 - 修改后包含字体加载
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 添加摄像机
    commands.spawn(Camera2d);
    
    // 加载字体文件（如果你有字体文件的话）
    // 将字体文件放在 assets/fonts/ 目录下
    let font = asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf");
    
    // 创建背景
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.6, 0.8)), // 蓝色背景代表学校
        BackgroundImage {
            name: "school".to_string(),
        },
    ));
    
    // 创建对话框UI
    commands.spawn((
        Node {
            width: Val::Percent(90.0),
            height: Val::Px(150.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(5.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        DialogueBox,
    ))
    .with_children(|parent| {
        // 角色名字 - 使用自定义字体
        parent.spawn((
            Text::new("角色名"),
            TextFont {
                font: font.clone(),  // 使用加载的字体
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
            CharacterName,
        ));
        
        // 对话文本 - 使用自定义字体
        parent.spawn((
            Text::new("按空格键开始对话..."),
            TextFont {
                font: font.clone(),  // 使用加载的字体
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            DialogueText,
        ));
    });
}

// 更新UI系统
fn update_ui_system(
    script_state: Res<ScriptState>,
    mut name_query: Query<&mut Text, (With<CharacterName>, Without<DialogueText>)>,
    mut text_query: Query<&mut Text, (With<DialogueText>, Without<CharacterName>)>,
    mut bg_query: Query<(&mut BackgroundColor, &BackgroundImage)>,
) {
    if script_state.current_index < script_state.nodes.len() {
        if let ScriptNode::Dialogue { character, text, background, .. } = &script_state.nodes[script_state.current_index] {
            // 更新角色名
            if let Ok(mut name_text) = name_query.single_mut() {
                name_text.0 = character.clone();
            }
            
            // 更新对话文本
            if let Ok(mut dialogue_text) = text_query.single_mut() {
                dialogue_text.0 = text.clone();
            }
            
            // 更新背景颜色（模拟背景切换）
            if let Some(bg_name) = background {
                for (mut bg_color, bg_image) in bg_query.iter_mut() {
                    if bg_image.name == *bg_name {
                        bg_color.0 = match bg_name.as_str() {
                            "school" => Color::srgb(0.2, 0.6, 0.8),
                            "library" => Color::srgb(0.4, 0.2, 0.6),
                            "park" => Color::srgb(0.2, 0.8, 0.2),
                            _ => Color::srgb(0.5, 0.5, 0.5), // 灰色
                        };
                    }
                }
            }
        }
    }
}

// 转场系统
fn transition_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TransitionEffect, &mut BackgroundColor), With<TransitionOverlay>>,
    mut script_state: ResMut<ScriptState>,
    mut bg_query: Query<(&mut BackgroundColor, &BackgroundImage), Without<TransitionOverlay>>,
) {
    for (entity, mut transition, mut overlay_color) in query.iter_mut() {
        transition.elapsed += time.delta_secs();
        let progress = (transition.elapsed / transition.duration).clamp(0.0, 1.0);
        
        match transition.transition_type {
            TransitionType::Fade => {
                let alpha = if progress < 0.5 {
                    progress * 2.0 // 先变黑
                } else {
                    2.0 - (progress * 2.0) // 再变透明
                };
                overlay_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
                
                // 中间时刻切换背景
                if progress >= 0.5 && transition.to_bg.is_some() {
                    change_background(&mut bg_query, &transition.to_bg);
                }
            },
            _ => {
                // 其他转场效果
                let alpha = 1.0 - progress;
                overlay_color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
            }
        }
        
        // 转场完成
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            script_state.is_transitioning = false;
            script_state.current_index += 1;
        }
    }
}

// 切换背景辅助函数
fn change_background(
    bg_query: &mut Query<(&mut BackgroundColor, &BackgroundImage), Without<TransitionOverlay>>,
    new_bg: &Option<String>,
) {
    if let Some(bg_name) = new_bg {
        for (mut bg_color, bg_image) in bg_query.iter_mut() {
            if bg_image.name == *bg_name {
                bg_color.0 = match bg_name.as_str() {
                    "school" => Color::srgb(0.2, 0.6, 0.8),
                    "library" => Color::srgb(0.4, 0.2, 0.6),
                    "park" => Color::srgb(0.2, 0.8, 0.2),
                    _ => Color::srgb(0.5, 0.5, 0.5), // 灰色
                };
            }
        }
    }
}

// 脚本执行系统
fn script_execution_system(
    mut commands: Commands,
    mut script_state: ResMut<ScriptState>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if script_state.is_transitioning {
        return;
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        if script_state.current_index < script_state.nodes.len() {
            let current_node = &script_state.nodes[script_state.current_index].clone();
            
            match current_node {
                ScriptNode::Dialogue { .. } => {
                    script_state.current_index += 1;
                },
                
                ScriptNode::Transition { transition, duration, from_bg, to_bg } => {
                    println!("执行转场：{} ({}秒)", transition, duration);
                    
                    // 创建转场遮罩
                    commands.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ZIndex(1000),
                        TransitionEffect {
                            transition_type: TransitionType::from(transition.as_str()),
                            duration: *duration,
                            elapsed: 0.0,
                            from_bg: from_bg.clone(),
                            to_bg: to_bg.clone(),
                        },
                        TransitionOverlay,
                    ));
                    
                    script_state.is_transitioning = true;
                },
            }
        }
    }
}

// 示例脚本数据
fn create_example_script() -> Vec<ScriptNode> {
    vec![
        ScriptNode::Dialogue {
            character: "黛安娜".to_string(),
            text: "我们去图书馆吧".to_string(),
            portrait: "protagonist".to_string(),
            background: Some("school".to_string()),
        },
        ScriptNode::Transition {
            transition: "fade".to_string(),
            duration: 2.0,
            from_bg: Some("school".to_string()),
            to_bg: Some("library".to_string()),
        },
        ScriptNode::Dialogue {
            character: "黛安娜".to_string(),
            text: "这里真安静呢".to_string(),
            portrait: "protagonist".to_string(),
            background: Some("library".to_string()),
        },
        ScriptNode::Transition {
            transition: "slide_left".to_string(),
            duration: 1.5,
            from_bg: Some("library".to_string()),
            to_bg: Some("park".to_string()),
        },
        ScriptNode::Dialogue {
            character: "黛安娜".to_string(),
            text: "公园的空气真清新".to_string(),
            portrait: "protagonist".to_string(),
            background: Some("park".to_string()),
        },
    ]
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ScriptState {
            nodes: create_example_script(),
            current_index: 0,
            is_transitioning: false,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            script_execution_system,
            update_ui_system,
            transition_system,
        ))
        .run();
}