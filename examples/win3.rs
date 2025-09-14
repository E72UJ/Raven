use bevy::prelude::*;
use bevy::window::{WindowMode, MonitorSelection};
use bevy::sprite::Anchor;

#[derive(Component)]
struct LeftAnchoredSprite;

#[derive(Component, Clone)]
enum MenuButton {
    StartGame,
    About,
    Settings,
    Help,
    Exit,
}

#[derive(Component)]
struct LeftAnchoredButton;

#[derive(Component)]
struct MenuButtonText;

#[derive(Component)]
struct FullscreenButton;

#[derive(Component)]
struct GameMenuOverlay;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input, 
            handle_button_click, 
            handle_menu_button_clicks,
            update_camera_scale,
            update_left_anchored_elements,
        ))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 相机
    commands.spawn((
        Camera2d,
        Transform::default(),
    ));
    
    // 背景图片
    commands.spawn((
        Sprite {
            image: asset_server.load("gui/main_menu.png"),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_translation(Vec3::new(-960.0, 0.0, 0.0)),
        LeftAnchoredSprite,
    ));
    
    // 创建游戏菜单叠加层（默认隐藏）
    commands.spawn((
        Sprite {
            image: asset_server.load("gui/overlay/game_menu.png"),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_translation(Vec3::new(-1080.0, 0.0, 0.1)),
        Visibility::Hidden, // 默认隐藏
        GameMenuOverlay, // 添加标记组件
    ));
    
    // UI 根节点
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    )).with_children(|parent| {
        // 全屏按钮
        parent.spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
            FullscreenButton,
        )).with_children(|button| {
            button.spawn(Text::new("esc"));
        });
        
        // 菜单按钮配置
        let button_texts = [
            ("开始游戏", MenuButton::StartGame),
            ("关于", MenuButton::About),
            ("设置", MenuButton::Settings),
            ("帮助", MenuButton::Help),
            ("退出", MenuButton::Exit),
        ];
        
        // 按钮样式配置
        let button_width = 200.0;
        let button_height = 60.0;
        let button_spacing = 50.0;
        let start_y = 150.0;
        let buttons_x = -100.0;
        
        // 创建菜单按钮
        for (i, (text, button_type)) in button_texts.iter().enumerate() {
            let y_position = 540.0 - start_y + (i as f32 * button_spacing);
            
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(button_width),
                    height: Val::Px(button_height),
                    position_type: PositionType::Absolute,
                    left: Val::Px(buttons_x),
                    top: Val::Px(y_position),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor(Color::NONE),
                button_type.clone(),
                LeftAnchoredButton,
            )).with_children(|button| {
                button.spawn((
                    Text::new(*text),
                    TextFont {
                        font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    MenuButtonText,
                ));
            });
        }
    });
}

fn handle_input(
    mut windows: Query<&mut Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F11) {
        toggle_fullscreen(&mut windows);
    }
}

// 更新左侧锚定元素的位置（包括按钮和图片）
fn update_left_anchored_elements(
    mut sprite_query: Query<&mut Transform, (With<LeftAnchoredSprite>, Without<Camera2d>)>,
    mut button_query: Query<&mut Node, With<LeftAnchoredButton>>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<LeftAnchoredSprite>)>,
) {
    if let (Ok(window), Ok(camera_transform)) = (windows.get_single(), camera_query.get_single()) {
        let camera_scale = camera_transform.scale.x;
        let effective_window_width = window.width() * camera_scale;
        
        // 更新背景图片位置
        for mut sprite_transform in &mut sprite_query {
            sprite_transform.translation.x = -effective_window_width / 2.0;
        }
        
        // 更新按钮位置 - 针对1920宽度时左移按钮组
        let base_buttons_x_offset = 210.0; // 基础偏移量
        
        // 当窗口宽度为1920时，额外向左移动
        let extra_left_offset = if window.width() >= 1920.0 {
            10.0 // 向左额外移动150像素，你可以调整这个值
        } else {
            0.0
        };
        
        let buttons_x_offset = base_buttons_x_offset - extra_left_offset;
        let ui_buttons_x = (960.0 - effective_window_width / 2.0) + buttons_x_offset;
        
        for mut button_node in &mut button_query {
            button_node.left = Val::Px(ui_buttons_x);
        }
    }
}

fn handle_button_click(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<FullscreenButton>)
    >,
    mut windows: Query<&mut Window>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.1, 0.1, 0.6));
                toggle_fullscreen(&mut windows);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.9));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.8));
            }
        }
    }
}

// 处理菜单按钮点击和hover效果
fn handle_menu_button_clicks(
    mut button_query: Query<(&Interaction, &MenuButton, &Children), (Changed<Interaction>, With<LeftAnchoredButton>)>,
    mut text_query: Query<&mut TextFont, With<MenuButtonText>>,
    mut overlay_query: Query<&mut Visibility, With<GameMenuOverlay>>, // 添加叠加层查询
    asset_server: Res<AssetServer>,
) {
    for (interaction, button_type, children) in &mut button_query {
        for child in children.iter() {
            if let Ok(mut text_font) = text_query.get_mut(child) {
                match *interaction {
                    Interaction::Pressed => {
                        text_font.font = asset_server.load("fonts/SarasaFixedHC-Regular.ttf");
                        
                        match button_type {
                            MenuButton::StartGame => {
                                println!("开始游戏被点击！");
                            },
                            MenuButton::About => {
                                println!("关于被点击！");
                                // 切换叠加层状态
                                if let Ok(mut visibility) = overlay_query.get_single_mut() {
                                    *visibility = match *visibility {
                                        Visibility::Visible => Visibility::Hidden,
                                        Visibility::Hidden => Visibility::Visible,
                                        Visibility::Inherited => Visibility::Visible, // 如果是继承状态，显示叠加层
                                    };
                                }
                            },
                            MenuButton::Settings => {
                                println!("设置被点击！");
                            },
                            MenuButton::Help => {
                                println!("帮助被点击！");
                            },
                            MenuButton::Exit => {
                                println!("退出被点击！");
                                std::process::exit(0);
                            },
                        }
                    },
                    Interaction::Hovered => {
                        text_font.font = asset_server.load("fonts/SarasaFixedHC-Regular.ttf");
                    },
                    Interaction::None => {
                        text_font.font = asset_server.load("fonts/SarasaFixedHC-Light.ttf");
                    },
                }
            }
        }
    }
}

fn toggle_fullscreen(windows: &mut Query<&mut Window>) {
    if let Ok(mut window) = windows.single_mut() {
        window.mode = match window.mode {
            WindowMode::Windowed => {
                println!("切换到无边框全屏");
                WindowMode::BorderlessFullscreen(MonitorSelection::Current)
            },
            _ => {
                println!("切换到窗口模式");
                WindowMode::Windowed
            },
        };
    }
}

fn update_camera_scale(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    windows: Query<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let (Ok(window), Ok(mut camera_transform)) = (windows.get_single(), camera_query.single_mut()) {
        let base_width = 1920.0;
        let base_height = 1080.0;
        let window_aspect = window.width() / window.height();
        let target_aspect = base_width / base_height;
        
        let scale = if keyboard.pressed(KeyCode::Digit1) {
            window.height() / base_height
        } else if keyboard.pressed(KeyCode::Digit2) {
            window.width() / base_width
        } else if keyboard.pressed(KeyCode::Digit3) {
            (window.width() / base_width).min(window.height() / base_height)
        } else {
            if window_aspect > target_aspect {
                window.height() / base_height
            } else {
                window.width() / base_width
            }
        };
        
        camera_transform.scale = Vec3::splat(1.0 / scale);
    }
}