use bevy::prelude::*;
use std::time::Duration;
use std::sync::atomic::{AtomicI32, Ordering};

// ========== 资源定义 ==========

#[derive(Resource, Default)]
pub struct MsgBoxManager {
    pub current_msgbox: Option<Entity>,
}

// 使用原子计数器代替可变静态变量
static COUNTER: AtomicI32 = AtomicI32::new(0);

// ========== 组件定义 ==========

#[derive(Component)]
pub struct MsgBox {
    pub timer: Timer,           // 控制整个生命周期
    pub fade_timer: Timer,      // 控制淡出效果
    pub is_fading: bool,        // 是否正在淡出
}

#[derive(Component)]
pub struct MsgBoxTypewriter {
    pub full_text: String,
    pub current_length: usize,
    pub timer: Timer,
    pub is_finished: bool,
}

#[derive(Component)]
pub struct MsgBoxText;

#[derive(Component)]
pub struct TestMsgBoxButton;

#[derive(Component)]
pub struct CustomFontButton;

#[derive(Component)]
pub struct FastTestButton;

impl MsgBoxTypewriter {
    pub fn new(text: String, chars_per_second: f32) -> Self {
        let delay = Duration::from_secs_f32(1.0 / chars_per_second);
        Self {
            full_text: text,
            current_length: 0,
            timer: Timer::new(delay, TimerMode::Repeating),
            is_finished: false,
        }
    }

    pub fn get_current_text(&self) -> String {
        self.full_text.chars().take(self.current_length).collect()
    }
}

impl MsgBox {
    pub fn new(display_duration_secs: f32, fade_duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(display_duration_secs, TimerMode::Once),
            fade_timer: Timer::from_seconds(fade_duration_secs, TimerMode::Once),
            is_fading: false,
        }
    }
}

// ========== MsgBox创建函数（修改后支持单实例） ==========

pub fn spawn_msgbox_with_font(
    commands: &mut Commands,
    asset_server: &AssetServer,
    msgbox_manager: &mut ResMut<MsgBoxManager>,
    message: String,
    font_path: Option<&str>, // 字体路径，None使用默认字体
    font_size: f32,
    typewriter_speed: f32,
    display_duration: f32,
    fade_duration: f32,
) -> Entity {
    // 先销毁现有的MsgBox（如果存在）
    if let Some(existing_entity) = msgbox_manager.current_msgbox {
        if let Ok(mut entity_commands) = commands.get_entity(existing_entity) {
            println!("销毁现有的MsgBox，实体ID: {:?}", existing_entity);
            entity_commands.despawn(); // 使用递归删除，确保子实体也被删除
        }
        msgbox_manager.current_msgbox = None;
    }

    let msgbox_entity = commands
        .spawn((
            Name::new("MsgBox"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(0.0),
                width: Val::Px(1280.0),
                height: Val::Px(185.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            BorderRadius::all(Val::Px(15.0)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            GlobalZIndex(15000),
            Visibility::Visible,
            MsgBox::new(display_duration, fade_duration),
        ))
        .with_children(|parent| {
            // 根据是否提供字体路径来设置字体
            let text_font = if let Some(font_path) = font_path {
                TextFont {
                    font: asset_server.load(font_path),
                    font_size,
                    ..default()
                }
            } else {
                TextFont {
                    font_size,
                    ..default()
                }
            };

            parent.spawn((
                Text::new(""),
                text_font,
                TextColor(Color::srgb(0.95, 0.95, 0.95)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                MsgBoxText,
                MsgBoxTypewriter::new(message, typewriter_speed),
            ));
        })
        .id();

    // 记录新创建的MsgBox
    msgbox_manager.current_msgbox = Some(msgbox_entity);
    println!("新MsgBox创建成功，实体ID: {:?}", msgbox_entity);
    
    msgbox_entity
}

// 更新便捷函数，添加字体参数
pub fn show_msgbox_with_font(
    commands: &mut Commands,
    asset_server: &AssetServer,
    msgbox_manager: &mut ResMut<MsgBoxManager>,
    message: &str,
    font_path: Option<&str>,
    font_size: Option<f32>,
) -> Entity {
    spawn_msgbox_with_font(
        commands,
        asset_server,
        msgbox_manager,
        message.to_string(),
        font_path,
        font_size.unwrap_or(28.0),
        25.0,  // 25字符/秒的打字速度
        3.0,   // 显示3秒
        1.5,   // 1.5秒淡出
    )
}

// 保持原有的函数作为默认选项
pub fn show_msgbox(
    commands: &mut Commands,
    asset_server: &AssetServer,
    msgbox_manager: &mut ResMut<MsgBoxManager>,
    message: &str,
) -> Entity {
    show_msgbox_with_font(commands, asset_server, msgbox_manager, message, None, None)
}

// 手动关闭当前MsgBox的函数
pub fn close_current_msgbox(
    commands: &mut Commands,
    msgbox_manager: &mut ResMut<MsgBoxManager>,
) {
    if let Some(existing_entity) = msgbox_manager.current_msgbox {
        if let Ok(mut entity_commands) = commands.get_entity(existing_entity) {
            println!("手动关闭当前MsgBox，实体ID: {:?}", existing_entity);
            entity_commands.despawn();
        }
        msgbox_manager.current_msgbox = None;
    }
}

// ========== 系统函数 ==========

// 打字机效果系统
pub fn msgbox_typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut MsgBoxTypewriter), With<MsgBoxText>>,
) {
    for (mut text, mut typewriter) in query.iter_mut() {
        if !typewriter.is_finished {
            typewriter.timer.tick(time.delta());

            if typewriter.timer.just_finished() {
                if typewriter.current_length < typewriter.full_text.chars().count() {
                    typewriter.current_length += 1;
                    text.0 = typewriter.get_current_text();
                } else {
                    typewriter.is_finished = true;
                    println!("打字机效果完成");
                }
            }
        }
    }
}

// MsgBox生命周期管理系统（修改后支持管理器清理）
pub fn msgbox_lifecycle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut msgbox_manager: ResMut<MsgBoxManager>,
    mut query: Query<(Entity, &mut MsgBox, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor, With<MsgBoxText>>,
    typewriter_query: Query<&MsgBoxTypewriter>,
) {
    for (entity, mut msgbox, mut bg_color, children) in query.iter_mut() {
        // 检查打字机是否完成
        let typewriter_finished = children
            .iter()
            .any(|child| {
                if let Ok(typewriter) = typewriter_query.get(child) {
                    typewriter.is_finished
                } else {
                    false
                }
            });

        // 只有在打字机完成后才开始倒计时
        if typewriter_finished {
            msgbox.timer.tick(time.delta());

            // 开始淡出阶段
            if msgbox.timer.finished() && !msgbox.is_fading {
                msgbox.is_fading = true;
                println!("MsgBox开始淡出");
            }

            // 淡出效果
            if msgbox.is_fading {
                msgbox.fade_timer.tick(time.delta());
                
                let fade_progress = msgbox.fade_timer.elapsed_secs() / msgbox.fade_timer.duration().as_secs_f32();
                let alpha = (1.0 - fade_progress.clamp(0.0, 1.0)).max(0.0);

                // 更新背景透明度 (保持原始颜色，只改变透明度)
                let mut new_bg = bg_color.0;
                new_bg.set_alpha(0.85 * alpha); // 0.85是原始透明度
                *bg_color = BackgroundColor(new_bg);

                // 更新文字透明度
                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        let mut new_text_color = text_color.0;
                        new_text_color.set_alpha(alpha);
                        text_color.0 = new_text_color;
                    }
                }

                // 完全淡出后销毁
                if msgbox.fade_timer.finished() {
                    println!("MsgBox淡出完成，销毁实体");
                    
                    // 从管理器中清除引用
                    if msgbox_manager.current_msgbox == Some(entity) {
                        msgbox_manager.current_msgbox = None;
                    }
                    
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// 默认字体按钮点击处理系统（修改后添加管理器参数）
pub fn handle_test_msgbox_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor), 
        (Changed<Interaction>, With<TestMsgBoxButton>)
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut msgbox_manager: ResMut<MsgBoxManager>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.2, 0.7, 0.2));
                
                // 使用默认字体
                show_msgbox(
                    &mut commands,
                    &asset_server,
                    &mut msgbox_manager,
                    "这是使用默认字体的消息框！如果之前有消息框，会被自动替换。"
                );
                
                println!("默认字体按钮被按下，MsgBox已创建");
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }
}

// 自定义字体按钮点击处理系统（修改后添加管理器参数）
pub fn handle_custom_font_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor), 
        (Changed<Interaction>, With<CustomFontButton>)
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut msgbox_manager: ResMut<MsgBoxManager>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.7, 0.2, 0.2));
                
                // 使用自定义字体（如果字体文件不存在，会fallback到默认字体）
                show_msgbox_with_font(
                    &mut commands,
                    &asset_server,
                    &mut msgbox_manager,
                    "这是使用自定义字体的消息框！字体大小也变大了。之前的消息框已被替换。",
                    Some("fonts/ark.ttf"), // 字体路径
                    Some(32.0), // 字体大小
                );
                
                println!("自定义字体按钮被按下，MsgBox已创建");
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.8, 0.4, 0.4));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.6, 0.3, 0.3));
            }
        }
    }
}

// 快速测试按钮处理系统（修正后使用原子计数器）
pub fn handle_fast_test_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor), 
        (Changed<Interaction>, With<FastTestButton>)
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut msgbox_manager: ResMut<MsgBoxManager>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.7));
                
                // 使用原子操作获取并增加计数器
                let count = COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
                let message = format!("快速切换测试 #{} - 每次点击都会立即替换之前的消息框！", count);
                
                show_msgbox_with_font(
                    &mut commands,
                    &asset_server,
                    &mut msgbox_manager,
                    &message,
                    None,
                    Some(26.0),
                );
                
                println!("快速测试按钮被按下，MsgBox已创建 (计数: {})", count);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.6));
            }
        }
    }
}

// 键盘快捷键系统（新增）
pub fn keyboard_shortcuts_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut msgbox_manager: ResMut<MsgBoxManager>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        show_msgbox(
            &mut commands,
            &asset_server,
            &mut msgbox_manager,
            "按空格键触发的消息！任何新消息都会替换当前显示的消息。"
        );
    }
    
    if keyboard.just_pressed(KeyCode::Escape) {
        close_current_msgbox(&mut commands, &mut msgbox_manager);
        println!("手动关闭消息框");
    }
}

// ========== UI设置函数（修改后添加快速测试按钮） ==========

pub fn setup_test_ui(mut commands: Commands) {
    // 默认字体按钮
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(50.0),
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        BorderRadius::all(Val::Px(8.0)),
        GlobalZIndex(1000),
        TestMsgBoxButton,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("默认字体"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    });

    // 自定义字体按钮
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(120.0),
            right: Val::Px(50.0),
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
        BorderRadius::all(Val::Px(8.0)),
        GlobalZIndex(1000),
        CustomFontButton,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("自定义字体"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    });

    // 快速测试按钮（新增）
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(190.0),
            right: Val::Px(50.0),
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.3, 0.6)),
        BorderRadius::all(Val::Px(8.0)),
        GlobalZIndex(1000),
        FastTestButton,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("快速切换"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
        ));
    });

    // 添加说明文字（修改后）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        Text::new("点击右下角的按钮来测试MsgBox组件\n\n- 灰色按钮：默认字体\n- 红色按钮：自定义字体（需要fonts/ark.ttf）\n- 蓝色按钮：快速切换测试\n\n键盘快捷键：\n- 空格键：显示测试消息\n- ESC键：手动关闭当前消息\n\n新消息会自动替换之前的消息框！"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        GlobalZIndex(500),
    ));

    println!("测试UI已创建");
}

// ========== 插件定义（修改后添加管理器和新系统） ==========

pub struct MsgBoxPlugin;

impl Plugin for MsgBoxPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MsgBoxManager>() // 初始化管理器资源
            .add_systems(Startup, setup_test_ui)
            .add_systems(
                Update, 
                (
                    msgbox_typewriter_system,
                    msgbox_lifecycle_system,
                    handle_test_msgbox_button,
                    handle_custom_font_button,
                    handle_fast_test_button, // 新增
                    keyboard_shortcuts_system, // 新增
                )
            );
    }
}

// ========== 主程序 ==========

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: "MsgBox Single Instance Test".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(MsgBoxPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    // UI相机
    commands.spawn(Camera2d);
    
    println!("相机已设置完成");
}