use bevy::prelude::*;
use bevy::window::{WindowMode, MonitorSelection};
use bevy::sprite::Anchor;

#[derive(Component)]
struct LeftAnchoredSprite;

#[derive(Component)]
struct MainMenuBackground;


#[derive(Component)]
struct BackButton;

#[derive(Component, Clone)]
enum MenuButton {
    StartGame,
    About,
    Settings,
    Help,
    Exit,
    Back,
}

#[derive(Component)]
struct LeftAnchoredButton;

#[derive(Component)]
struct MenuButtonText;

#[derive(Component)]
struct FullscreenButton;

#[derive(Component)]
struct GameMenuOverlay;

#[derive(Component)]
struct AboutTitle;

#[derive(Component)]
struct AboutContent;

#[derive(Component)]
struct HelpTitle;

#[derive(Component)]
struct HelpContent;

#[derive(Component)]
struct RightAnchoredContent;

#[derive(Component)]
struct SettingsTitle;

#[derive(Component)]
struct SettingsContent;

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
            update_right_anchored_elements,
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
        Transform::from_translation(Vec3::new(-960.0, 0.0, 0.2)),
        LeftAnchoredSprite,
        MainMenuBackground,
    ));
    commands.spawn((
        Sprite {
            image: asset_server.load("gui/game3.png"),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_translation(Vec3::new(-960.0, 0.0, 0.1)),
    ));
    
    // 创建游戏菜单叠加层（默认隐藏）
    commands.spawn((
        Sprite {
            image: asset_server.load("gui/overlay/game_menu.png"),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            anchor: Anchor::CenterLeft,
            ..default()
        },
        Transform::from_translation(Vec3::new(-960.0, 0.0, 0.3)),
        Visibility::Hidden,
        GameMenuOverlay,
    ));
    // 添加返回按钮

    // UI 根节点
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    }).with_children(|parent| {
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
        
        // 关于标题（默认隐藏）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                ..default()
            },
            Visibility::Hidden,
            AboutTitle,
        )).with_children(|title_parent| {
            title_parent.spawn((
                Text::new("关于"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
        
        // 帮助标题（默认隐藏）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                ..default()
            },
            Visibility::Hidden,
            HelpTitle,
        )).with_children(|title_parent| {
            title_parent.spawn((
                Text::new("帮助"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
        
        // 设置标题（默认隐藏）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(20.0),
                ..default()
            },
            Visibility::Hidden,
            SettingsTitle,
        )).with_children(|title_parent| {
            title_parent.spawn((
                Text::new("设置"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
        
        // 关于页面内容（默认隐藏，右侧显示）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(300.0),
                width: Val::Px(500.0),
                height: Val::Px(600.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
            Visibility::Hidden,
            AboutContent,
            RightAnchoredContent,
        )).with_children(|about_parent| {
            about_parent.spawn((
                Text::new("我的视觉小说游戏"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            about_parent.spawn((
                Text::new("版本 0.1.3"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));
            about_parent.spawn((
                Text::new("本案例使用 Raven engine 制作"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
        
        // 帮助页面内容（默认隐藏，右侧显示）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(300.0),
                width: Val::Px(500.0),
                height: Val::Px(600.0),
                padding: UiRect::all(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
            Visibility::Hidden,
            HelpContent,
            RightAnchoredContent,
        )).with_children(|help_parent| {
            help_parent.spawn((
                Text::new("帮助"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
        // 添加游戏操作说明
        help_parent.spawn((
            Text::new("游戏操作说明："),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // 回退上一句操作
        help_parent.spawn((
            Text::new("回退上一句：        ←"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        // 进入下一句操作
        help_parent.spawn((
            Text::new("进入下一句：        Enter"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));

        // 退出主界面操作
        help_parent.spawn((
            Text::new("退出主界面：        ESC"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
        ));

        // 添加额外的帮助信息
        help_parent.spawn((
            Text::new("游戏提示："),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        help_parent.spawn((
            Text::new("• 使用方向键可以控制游戏进度"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        help_parent.spawn((
            Text::new("• 按 ESC 键可以随时返回主菜单"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        help_parent.spawn((
            Text::new("• 支持键盘快捷键操作"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));
        });
        
        // 设置页面内容（默认隐藏）
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                left: Val::Px(300.0),
                width: Val::Px(900.0),
                height: Val::Px(600.0),
                padding: UiRect::all(Val::Px(30.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            Visibility::Hidden,
            SettingsContent,
        )).with_children(|settings_parent| {
            // 左侧列 - 显示选项
            settings_parent.spawn(Node {
                width: Val::Px(300.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            }).with_children(|left_column| {
                // 显示部分标题
                left_column.spawn((
                    Text::new("显示"),
                    TextFont {
                        font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.2)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));
                
                // 显示选项
                let display_options = ["窗口", "固定大小"];
                for option in display_options {
                    left_column.spawn((
                        Text::new(option),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            });
            
            // 中间列 - 快进选项
            settings_parent.spawn(Node {
                width: Val::Px(300.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            }).with_children(|middle_column| {
                // 快进部分标题
                middle_column.spawn((
                    Text::new("快进"),
                    TextFont {
                        font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.2)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));
                
                let skip_options = ["未读文本", "选项后继续", "忽略转场"];
                for option in skip_options {
                    middle_column.spawn((
                        Text::new(option),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                }
            });
            
            // 右侧列 - 语言选项
            settings_parent.spawn(Node {
                width: Val::Px(400.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            }).with_children(|right_column| {
                // 语言部分标题
                right_column.spawn((
                    Text::new("语言"),
                    TextFont {
                        font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.2)),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));
                
                // 创建两列语言选项
                right_column.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }).with_children(|lang_row| {
                    // 左语言列
                    lang_row.spawn(Node {
                        width: Val::Percent(48.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }).with_children(|lang_left| {
                        let left_languages = ["English", "Český", "Dansk", "Français", "Italiano", "Bahasa Melayu", "Русский"];
                        for lang in left_languages {
                            lang_left.spawn((
                                Text::new(lang),
                                TextFont {
                                    font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                                Node {
                                    margin: UiRect::bottom(Val::Px(8.0)),
                                    ..default()
                                },
                            ));
                        }
                    });
                    
                    // 右语言列
                    lang_row.spawn(Node {
                        width: Val::Percent(48.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }).with_children(|lang_right| {
                        let right_languages = ["Español", "Українська", "日本語", "한국어", "简体中文", "繁體中文"];
                        for lang in right_languages.iter() {
                            let color = if lang == &"简体中文" {
                                Color::srgb(1.0, 0.6, 0.2) // 橙色高亮
                            } else {
                                Color::srgb(0.8, 0.8, 0.8)
                            };
                            
                            lang_right.spawn((
                                Text::new(*lang),
                                TextFont {
                                    font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(color),
                                Node {
                                    margin: UiRect::bottom(Val::Px(8.0)),
                                    ..default()
                                },
                            ));
                        }
                    });
                });
            });
        });
        
        // 菜单按钮配置
        let button_texts = [
            ("开始游戏", MenuButton::StartGame),
            ("关于", MenuButton::About),
            ("设置", MenuButton::Settings),
            ("帮助", MenuButton::Help),
            ("返回", MenuButton::Back),
            ("退出", MenuButton::Exit),

        ];
        
        let button_width = 200.0;
        let button_height = 60.0;
        let button_spacing = 50.0;
        let start_y = 300.0;
        let buttons_x = 20.0;

        for (i, (text, button_type)) in button_texts.iter().enumerate() {
            let y_position = 540.0 - start_y + (i as f32 * button_spacing);
        // 根据按钮类型确定初始可见性
            let initial_visibility = match button_type {
                MenuButton::Back => Visibility::Visible,  // 返回按钮默认隐藏
                _ => Visibility::Visible,                // 其他按钮默认可见
            };
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
                initial_visibility,  // 设置初始可见性
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

fn handle_menu_button_clicks(
    mut button_query: Query<(&Interaction, &MenuButton, &Children), (Changed<Interaction>, With<LeftAnchoredButton>)>,
    mut text_query: Query<&mut TextFont, With<MenuButtonText>>,
    mut visibility_query: ParamSet<(
        Query<&mut Visibility, With<GameMenuOverlay>>,
        Query<&mut Visibility, With<MainMenuBackground>>,
        Query<&mut Visibility, With<AboutTitle>>,
        Query<&mut Visibility, With<AboutContent>>,
        Query<&mut Visibility, With<HelpTitle>>,
        Query<&mut Visibility, With<HelpContent>>,
        Query<&mut Visibility, With<SettingsTitle>>,
        Query<&mut Visibility, With<SettingsContent>>,
    )>,
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
                              // 显示返回按钮
                            },
                            MenuButton::About => {
                                println!("关于被点击！");
                                // 显示关于页面，隐藏其他页面
                                if let Ok(mut overlay) = visibility_query.p0().single_mut() {
                                    *overlay = Visibility::Visible;
                                }
                                if let Ok(mut main_menu) = visibility_query.p1().single_mut() {
                                    *main_menu = Visibility::Hidden;
                                }
                                if let Ok(mut about_title) = visibility_query.p2().single_mut() {
                                    *about_title = Visibility::Visible;
                                }
                                if let Ok(mut about_content) = visibility_query.p3().single_mut() {
                                    *about_content = Visibility::Visible;
                                }
                                if let Ok(mut help_title) = visibility_query.p4().single_mut() {
                                    *help_title = Visibility::Hidden;
                                }
                                if let Ok(mut help_content) = visibility_query.p5().single_mut() {
                                    *help_content = Visibility::Hidden;
                                }
                                if let Ok(mut settings_title) = visibility_query.p6().single_mut() {
                                    *settings_title = Visibility::Hidden;
                                }
                                if let Ok(mut settings_content) = visibility_query.p7().single_mut() {
                                    *settings_content = Visibility::Hidden;
                                }
                            },
                            MenuButton::Settings => {
                                println!("设置被点击！");
                                // 显示设置页面，隐藏其他页面
                                if let Ok(mut overlay) = visibility_query.p0().single_mut() {
                                    *overlay = Visibility::Visible;
                                }
                                if let Ok(mut main_menu) = visibility_query.p1().single_mut() {
                                    *main_menu = Visibility::Hidden;
                                }
                                if let Ok(mut about_title) = visibility_query.p2().single_mut() {
                                    *about_title = Visibility::Hidden;
                                }
                                if let Ok(mut about_content) = visibility_query.p3().single_mut() {
                                    *about_content = Visibility::Hidden;
                                }
                                if let Ok(mut help_title) = visibility_query.p4().single_mut() {
                                    *help_title = Visibility::Hidden;
                                }
                                if let Ok(mut help_content) = visibility_query.p5().single_mut() {
                                    *help_content = Visibility::Hidden;
                                }
                                if let Ok(mut settings_title) = visibility_query.p6().single_mut() {
                                    *settings_title = Visibility::Visible;
                                }
                                if let Ok(mut settings_content) = visibility_query.p7().single_mut() {
                                    *settings_content = Visibility::Visible;
                                }
                            },
                            MenuButton::Help => {
                                println!("帮助被点击！");
                                // 显示帮助页面，隐藏其他页面
                                if let Ok(mut overlay) = visibility_query.p0().single_mut() {
                                    *overlay = Visibility::Visible;
                                }
                                if let Ok(mut main_menu) = visibility_query.p1().single_mut() {
                                    *main_menu = Visibility::Hidden;
                                }
                                if let Ok(mut about_title) = visibility_query.p2().single_mut() {
                                    *about_title = Visibility::Hidden;
                                }
                                if let Ok(mut about_content) = visibility_query.p3().single_mut() {
                                    *about_content = Visibility::Hidden;
                                }
                                if let Ok(mut help_title) = visibility_query.p4().single_mut() {
                                    *help_title = Visibility::Visible;
                                }
                                if let Ok(mut help_content) = visibility_query.p5().single_mut() {
                                    *help_content = Visibility::Visible;
                                }
                                if let Ok(mut settings_title) = visibility_query.p6().single_mut() {
                                    *settings_title = Visibility::Hidden;
                                }
                                if let Ok(mut settings_content) = visibility_query.p7().single_mut() {
                                    *settings_content = Visibility::Hidden;
                                }
                            },
                            MenuButton::Exit => {
                                println!("退出被点击！");
                                std::process::exit(0);
                            },
                            MenuButton::Back => {
                                println!("返回被点击！回到主菜单");
                                // 隐藏遮罩
                                if let Ok(mut overlay) = visibility_query.p0().single_mut() {
                                    *overlay = Visibility::Hidden;
                                }
                                // 显示主菜单背景
                                if let Ok(mut main_menu) = visibility_query.p1().single_mut() {
                                    *main_menu = Visibility::Visible;
                                }
                                // 隐藏所有页面内容
                                if let Ok(mut about_title) = visibility_query.p2().single_mut() {
                                    *about_title = Visibility::Hidden;
                                }
                                if let Ok(mut about_content) = visibility_query.p3().single_mut() {
                                    *about_content = Visibility::Hidden;
                                }
                                if let Ok(mut help_title) = visibility_query.p4().single_mut() {
                                    *help_title = Visibility::Hidden;
                                }
                                if let Ok(mut help_content) = visibility_query.p5().single_mut() {
                                    *help_content = Visibility::Hidden;
                                }
                                if let Ok(mut settings_title) = visibility_query.p6().single_mut() {
                                    *settings_title = Visibility::Hidden;
                                }
                                if let Ok(mut settings_content) = visibility_query.p7().single_mut() {
                                    *settings_content = Visibility::Hidden;
                                }
                            
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

fn handle_input(
    mut windows: Query<&mut Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F11) {
        toggle_fullscreen(&mut windows);
    }
}

fn update_left_anchored_elements(
    mut sprite_query: Query<&mut Transform, (With<LeftAnchoredSprite>, Without<Camera2d>)>,
    mut button_query: Query<&mut Node, With<LeftAnchoredButton>>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<LeftAnchoredSprite>)>,
) {
    if let (Ok(window), Ok(camera_transform)) = (windows.get_single(), camera_query.get_single()) {
        let camera_scale = camera_transform.scale.x;
        let effective_window_width = window.width() * camera_scale;
        
        for mut sprite_transform in &mut sprite_query {
            sprite_transform.translation.x = -effective_window_width / 2.0;
        }
        
        let base_buttons_x_offset = 0.0;
        let extra_left_offset = if window.width() >= 1920.0 {
            10.0
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

fn update_right_anchored_elements(
    mut content_query: Query<&mut Node, With<RightAnchoredContent>>,
    windows: Query<&Window>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    if let (Ok(window), Ok(camera_transform)) = (windows.get_single(), camera_query.get_single()) {
        let camera_scale = camera_transform.scale.x;
        let effective_window_width = window.width() * camera_scale;
        
        let base_right_offset = 200.0;
        let extra_right_offset = if window.width() >= 1920.0 {
            20.0
        } else {
            0.0
        };
        
        let right_offset = base_right_offset + extra_right_offset;
        let ui_right_x = (effective_window_width / 2.0 - 960.0) + right_offset;
        
        for mut content_node in &mut content_query {
            content_node.right = Val::Px(ui_right_x);
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