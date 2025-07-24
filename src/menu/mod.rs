// src/menu/mod.rs
use bevy::{input_focus::InputFocus, prelude::*, winit::WinitSettings};
use crate::GameScene;

use crate::style::UiStyleSheet;
use crate::style::StylePlugin;

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
struct SettingsEntity;

#[derive(Component)]
struct MenuCamera;

pub struct MenuPlugin;



impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        
            // 只有在有用户输入时才运行应用程序，这将显著减少CPU/GPU使      用
            // .insert_resource(WinitSettings::desktop_app())
            // 必须设置 `InputFocus` 以便辅助功能识别按钮
            .init_resource::<InputFocus>()
            // 删除这行！不要重复初始化状态 - main.rs 已经初始化了
            // .init_state::<GameScene>()
            .add_systems(Startup, setup)
            // 重要：只在 Menu 状态下运行按钮系统
            .add_systems(Update, button_system.run_if(in_state(GameScene::Menu)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::About)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::Help)))
            .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
            // 只管理菜单场景
            .add_systems(OnEnter(GameScene::Menu), setup_menu_scene)
            .add_systems(OnExit(GameScene::Menu), cleanup_all_menu)
            .add_systems(OnEnter(GameScene::Settings), setup_settings_overlay)
            .add_systems(OnExit(GameScene::Settings), cleanup_scene)
            .add_systems(OnEnter(GameScene::About), setup_about_scene)
            .add_systems(OnExit(GameScene::About), cleanup_all_about)
            .add_systems(OnEnter(GameScene::Help), setup_help_scene)
            .add_systems(OnExit(GameScene::Help), cleanup_all_about);

    }
}

// 组件定义（只保留菜单相关的）
#[derive(Component)]
pub struct StartGameButton;

#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct LoadGameButton;

#[derive(Component)]
pub struct HelpButton;

#[derive(Component)]
pub struct AboutButton;

#[derive(Component)]
pub struct ExitGameButton;

#[derive(Component)]
struct SceneEntity;

#[derive(Component)]
pub struct AboutUI;

#[derive(Component)]
pub struct BackToMenuButton;



// 按钮颜色常量
// const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);

const NORMAL_BUTTON: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);
fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameScene>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
            Option<&StartGameButton>,
            Option<&SettingsButton>,
            Option<&ExitGameButton>,
            Option<&AboutButton>,  
            Option<&BackButton>,  
            Option<&HelpButton>,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children, start_game, settings, exit_game,about,back,help) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *border_color = BorderColor(Color::WHITE.with_alpha(0.8));
                *color = PRESSED_BUTTON.into();
                button.set_changed();

                // 根据按钮类型处理场景切换
                if start_game.is_some() {
                    next_state.set(GameScene::Game);
                    println!("状态改变了");
                } else if settings.is_some() {
                    next_state.set(GameScene::Settings);
                } else if about.is_some() {
                    next_state.set(GameScene::About);
                } else if back.is_some() {  // 添加返回按钮处理
                    next_state.set(GameScene::Menu);
                }else if help.is_some() {
                    next_state.set(GameScene::Help);
                }  else if exit_game.is_some() {
                    std::process::exit(0);
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *border_color = BorderColor(Color::WHITE.with_alpha(0.6));
                *color = HOVERED_BUTTON.into();
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *border_color = BorderColor(Color::WHITE);
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn setup(mut commands: Commands) {
    // UI 摄像机
    commands.spawn((Camera2d, MenuCamera)); // 添加标记组件
}

fn setup_menu_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        children![
            // 左侧菜单区域
            (
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect {
                        left: Val::Px(50.0),
                        right: Val::Px(30.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(70.0),
                    },
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                children![
                    (
                        Text::new("Raven demo"),
                        TextFont {
                            font: assets.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                        GlobalZIndex(99)
                    ),
                    create_button(&assets, "开始游戏", StartGameButton),
                    create_button(&assets, "载入存档", LoadGameButton),
                    create_button(&assets, "设置", SettingsButton),
                    create_button(&assets, "关于", AboutButton),
                    create_button(&assets, "帮助", HelpButton),
                    create_button(&assets, "退出", ExitGameButton),
                ],
            ),
            // 右侧图片区域
            (
                Node {
                    // height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    width: Val::Px(1280.0),
                    height: Val::Px(720.0),
                    left: Val::Px(00.0), // 向左偏移50像素，可以调整数值
                    top: Val::Px(-2.0), // 向左偏移50像素，可以调整数值    
                    // justify_content: JustifyContent::Center,
                    // align_items: AlignItems::Center,
                    ..default()
                },
    children![(
        // 下层sprite，可以自定义位置和大小
        ImageNode::new(assets.load("gui/main_menu.png")),
        Node {
            position_type: PositionType::Absolute,
            // left: Val::Px(100.0),       // 相对位置
            // top: Val::Px(50.0),         // 相对位置
            ..default()
        },
        GlobalZIndex(-1),
    ), (
        // 原有的overlay图片
        ImageNode::new(assets.load("gui/overlay_main_menu.png")).with_color(Color::hsl(0.6, 2.0, 1.0)),
        Node {
            // width: Val::Px(1157.0),
            // height: Val::Px(650.0),
            ..default()
        },
        GlobalZIndex(0),
    )],
            ),
        ],
    ));
}

fn create_button(asset_server: &AssetServer, text: &str, button_type: impl Component) -> impl Bundle {
    (
        button_type,
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(20.0),
            // border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // BorderColor(Color::WHITE),
        // BackgroundColor(None),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // 完全透明
        GlobalZIndex(55),
        children![(
            Text::new(text),
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 20.0,
                ..default()
            },
            // TextColor(Color::srgb(0.9, 0.9, 0.9)),
            
            // TextShadow::default(),
        )]
    )
}

fn cleanup_scene(
    mut commands: Commands, 
    scene_query: Query<Entity, With<SceneEntity>>,
    camera_query: Query<Entity, With<MenuCamera>>,
    current_state: Res<State<GameScene>>,
) {
    println!("清理场景，当前状态: {:?}", current_state.get());
    
    for entity in &scene_query {
        commands.entity(entity).despawn();
    }
    
    // 只有进入Game状态时才清理摄像机
    if *current_state.get() == GameScene::Game {
        println!("进入游戏状态，清理菜单摄像机");
        for entity in &camera_query {
            commands.entity(entity).despawn();
        }
    } else {
        println!("保留摄像机用于其他菜单场景");
    }
}
fn cleanup_all_menu(
    mut commands: Commands,
    scene_query: Query<Entity, With<SceneEntity>>,
    camera_query: Query<Entity, With<MenuCamera>>,
) {
    println!("进入游戏，清理所有菜单元素");
    
    for entity in &scene_query {
        commands.entity(entity).despawn();
    }
    for entity in &camera_query {
        commands.entity(entity).despawn();
    }
}
fn cleanup_all_about(
    mut commands: Commands,
    about_ui_query: Query<Entity, With<AboutUI>>,
) {
    println!("清理关于界面");
    
    for entity in &about_ui_query {
        commands.entity(entity).despawn_recursive();
    }

}
fn setup_about_scene(mut commands: Commands, asset_server: Res<AssetServer>,camera_query: Query<Entity, With<MenuCamera>>) {
    println!("{}","执行关于界面");
    if camera_query.is_empty() {
        // 如果没有,则创建一个新的菜单摄像机
        commands.spawn((Camera2d, MenuCamera));
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            AboutUI,
        ))
        .with_children(|parent| {
            // 关于窗口
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    BorderColor(Color::srgb(0.6, 0.6, 0.8)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn(Text::new("关于"));

                    // 游戏信息容器
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text::new("Raven engine v0.1.5"));
                            parent.spawn(Text::new("开发者：Furau"));
                            parent.spawn(Text::new("双模架构轻量级视觉小说引擎"));
                            parent.spawn(Text::new("这是一个使用Raven开发的游戏。感谢您的游玩！"));
                        });

                    // 返回按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(45.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                            BorderColor(Color::srgb(0.5, 0.5, 0.7)),
                            BackButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn(Text::new("返回"));
                        });
                });
        });
}

fn setup_help_scene(mut commands: Commands, asset_server: Res<AssetServer>,camera_query: Query<Entity, With<MenuCamera>>) {
    println!("{}","执行帮助界面");
    if camera_query.is_empty() {
        // 如果没有,则创建一个新的菜单摄像机
        commands.spawn((Camera2d, MenuCamera));
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            AboutUI,
        ))
        .with_children(|parent| {
            // 关于窗口
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    BorderColor(Color::srgb(0.6, 0.6, 0.8)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn(Text::new("帮助"));

                    // 游戏信息容器
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(Text::new("回退上一句:   ←  "));
                            parent.spawn(Text::new("进入下一句: Enter"));
                            parent.spawn(Text::new("退出主界面:  ESC "));
                            parent.spawn(Text::new("感谢您使用本引擎，任何问题可以电邮至Furau@qq.com"));
                        });

                    // 返回按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(45.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                            BorderColor(Color::srgb(0.5, 0.5, 0.7)),
                            BackButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn(Text::new("返回"));
                        });
                });
        });
}

fn setup_settings_overlay(mut commands: Commands, asset_server: Res<AssetServer>, camera_query: Query<Entity, With<MenuCamera>>) {
    println!("在主菜单基础上添加设置界面...");

    // 首先检查是否已经有菜单摄像机存在
    if camera_query.is_empty() {
        // 如果没有,则创建一个新的菜单摄像机
        commands.spawn((Camera2d, MenuCamera));
    }

    // 创建半透明遮罩背景
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.7),
            custom_size: Some(Vec2::new(880.0, 720.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        SettingsEntity,
    ));

    // 设置界面主体
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(400.0),
            height: Val::Px(300.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect::new(Val::Px(-200.0), Val::Auto, Val::Px(-150.0), Val::Auto),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        BorderColor(Color::WHITE),
        SettingsEntity,
        children![
            // 标题
            (
                Text::new("设置"),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
            // 音量设置
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },
                children![
                    (
                        Text::new("音量"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ),
                ],
            ),
            // 文字速度设置
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },
                children![
                    (
                        Text::new("文字速度"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ),
                ],
            ),
            // 全屏设置
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    ..default()
                },
                children![
                    (
                        Text::new("全屏"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ),
                    (
                        Button,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(30.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                        children![(
                            Text::new("关"),
                            TextFont {
                                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        )],
                    ),
                ],
            ),
            // 关闭按钮
            (
                Button,
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                children![(
                    Text::new("关闭"),
                    TextFont {
                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ),
        ],
    ));
}
