use bevy::{input_focus::InputFocus, prelude::*,window::{WindowResized,Window,PrimaryWindow},text::TextColor};

use crate::GameScene;
use crate::audio::{AudioManager,stop_all_audio,stop_all_audio_system};
use crate::style::{UiStyleSheet, load_styles}; 
use crate::config::MainConfig;
use crate::url::{UrlButton,open_url};


#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
struct SettingsEntity;

#[derive(Component)]
struct MenuCamera;

// 在你的组件定义中添加
#[derive(Component)]
struct GameMenuOverlay;

#[derive(Component)]
struct MainMenuBackground;


pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<InputFocus>()
            .init_resource::<UiStyleSheet>()
            // .init_state::<GameScene>()
            .add_systems(Startup, setup)
            .add_systems(Update, button_system.run_if(in_state(GameScene::LoadButton)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::Settings)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::Menu)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::About)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::Help)))
            .add_systems(Update, button_system.run_if(in_state(GameScene::GameSettings)))
            .add_systems(Update, update_background_size_on_resize)
            .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
            
            .add_systems(OnEnter(GameScene::LoadButton), setup_load_scene)     // 调用载入场景设置函数
            .add_systems(OnExit(GameScene::LoadButton), cleanup_load_scene)    // 调用载入场景清理函数
            // .add_systems(OnEnter(GameScene::Menu), (load_styles, setup_menu_scene).chain())
            .add_systems(OnEnter(GameScene::Menu), load_styles)
            .add_systems(OnEnter(GameScene::Menu), setup_menu_scene.after(load_styles))
            .add_systems(OnExit(GameScene::Menu), (on_exit_game_state,stop_all_audio_system))
            .add_systems(OnEnter(GameScene::Settings), setup_settings_overlay)
            .add_systems(OnExit(GameScene::Settings), cleanup_settings_overlay)
            .add_systems(OnEnter(GameScene::About), setup_about_scene)
            .add_systems(OnExit(GameScene::About), cleanup_all_about)
            .add_systems(OnEnter(GameScene::Help), setup_help_scene)
            .add_systems(OnExit(GameScene::Help), cleanup_all_about)

        // 主游戏状态
            .add_systems(OnEnter(GameScene::Game), cleanup_for_game)
            .add_systems(OnExit(GameScene::Game), cleanup_cameras)

            .add_systems(OnEnter(GameScene::GameSettings), setup_game_settings_overlay)
            .add_systems(OnExit(GameScene::GameSettings), cleanup_game_settings_overlay);

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
pub struct LoadButton;

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
pub struct LoadUI;

#[derive(Component)]
pub struct BackToMenuButton;



// 按钮颜色常量
// const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);

const NORMAL_BUTTON: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_BUTTON: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

// 按钮交互颜色
const NORMAL_BUTTON_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
const HOVERED_BUTTON_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);
const PRESSED_BUTTON_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

const NORMAL_BUTTON_FONT: &str = "fonts/SarasaFixedHC-Light.ttf";
const HOVERED_BUTTON_FONT: &str = "fonts/SarasaFixedHC-Regular.ttf";

fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameScene>>,
    asset_server: Res<AssetServer>,
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
            Option<&LoadGameButton>,
            Option<&UrlButton>
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut TextFont>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children, start_game, settings, exit_game, about, back, help, load, url_button) in
        &mut interaction_query
    {
        if let Ok(mut text_font) = text_query.get_mut(children[0]) {
            match *interaction {
                Interaction::Pressed => {
                    input_focus.set(entity);
                    button.set_changed();
                    text_font.font = asset_server.load(HOVERED_BUTTON_FONT);

                    // 根据按钮类型处理场景切换
                    if start_game.is_some() {
                        next_state.set(GameScene::Game);
                    } else if settings.is_some() {
                        next_state.set(GameScene::Settings);
                    } else if about.is_some() {
                        next_state.set(GameScene::About);
                    } else if back.is_some() {  
                        next_state.set(GameScene::Menu);
                    } else if help.is_some() {
                        next_state.set(GameScene::Help);
                    } else if load.is_some() {  
                        next_state.set(GameScene::LoadButton);
                    } else if exit_game.is_some() {
                        std::process::exit(0);
                    } else if let Some(url_btn) = url_button {
                        println!("打开URL: {}", url_btn.url);
                        if let Err(e) = open_url(&url_btn.url) {
                            println!("打开链接时出错: {}", e);
                        }
                    }
                }
                Interaction::Hovered => {
                    input_focus.set(entity);
                    *border_color = BorderColor::from(Color::WHITE.with_alpha(0.6));
                    button.set_changed();
                    text_font.font = asset_server.load(HOVERED_BUTTON_FONT);
                }
                Interaction::None => {
                    input_focus.clear();
                    *border_color = BorderColor::from(Color::WHITE);
                    text_font.font = asset_server.load(NORMAL_BUTTON_FONT);
                }
            }
        }
    }
}

fn setup(mut commands: Commands) {
    // UI 摄像机
    commands.spawn((Camera2d, MenuCamera)); // 添加标记组件
    
}

fn setup_menu_scene(
    mut commands: Commands, 
    assets: Res<AssetServer>,
    style_sheet: Res<UiStyleSheet>,
    config: Res<MainConfig>,
    scene_query: Query<Entity, With<SceneEntity>>,
) {
    // 检查是否已经创建了场景实体
    if !scene_query.is_empty() {
        println!("场景实体已经存在，跳过创建");
        return;
    }

    // 样式渲染
    let logo_font_size = style_sheet.get_font_size("menu", "logo");
    let logo_text_color = style_sheet.get_text_color("menu", "logo");
    let logo_position = style_sheet.get_position("menu", "logo");
    let menu_game_main_size: Option<(Val, Val)> = style_sheet.get_size("menu", "menu_game_menu");
    let logo_text = &config.settings.logo_text;

    // ===== Sprite背景层 =====
    // 主背景图片
    commands.spawn((
        Sprite {
            image: assets.load("gui/main_menu.png"),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-0.0, 0.0, 0.1)),
        SceneEntity,
    ));

    // 游戏内容背景图片
    commands.spawn((
        Sprite {
            image: assets.load("gui/game3.png"),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
        SceneEntity,
    ));

    // 菜单覆盖层（可选显示/隐藏）
    commands.spawn((
        Name::new("menu_overlay"),
        Sprite {
            image: assets.load("gui/overlay_main_menu.png"),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            // color: Color::hsl(0.6, 2.0, 1.0), // 应用颜色滤镜
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.25)),
        SceneEntity,
    ));

    // 游戏菜单叠加层（默认隐藏，用于游戏内菜单）
    commands.spawn((
        Name::new("game_menu_overlay"),
        Sprite {
            image: assets.load("gui/overlay/game_menu.png"),
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.3)),
        Visibility::Hidden,
        SceneEntity,
        GameMenuOverlay
    ));

    // ===== UI菜单层 =====
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        // 使用高Z-index确保UI在Sprite之上
        GlobalZIndex(100),
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
                    // Logo文本
                    (
                        Text::new(logo_text),
                        TextFont {
                            font: assets.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: logo_font_size,
                            ..default()
                        },
                        TextColor(logo_text_color),
                        Node {
                            margin: UiRect {
                                left: Val::Px(20.0),
                                right: Val::Px(0.0),
                                top: Val::Px(0.0),
                                bottom: Val::Px(0.0),
                            },
                            ..default()
                        },
                        GlobalZIndex(110), // 比菜单容器更高
                    ),
                    // 菜单按钮
                    create_button(&assets, "开始游戏", StartGameButton),
                    create_button(&assets, "关于", AboutButton),
                    create_button(&assets, "帮助", HelpButton),
                    create_button(&assets, "退出", ExitGameButton),
                ],
            ),
            // 右侧透明区域（用于保持布局但不遮挡背景）
            (
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    // 不设置背景，保持透明
                    ..default()
                },
            ),
        ],
    ));
}
fn create_button(asset_server: &AssetServer, text: &str, button_type: impl Component) -> impl Bundle {
    
    (
        button_type,
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(35.0),
            // border: UiRect::all(Val::Px(2.0)),
            margin: UiRect {
                left: Val::Px(14.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        // BorderColor::all(Color::WHITE),
        // BackgroundColor(None),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // 完全透明
        GlobalZIndex(55),
        children![(
            Text::new(text),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 26.0,
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
// fn cleanup_all_menu(
//     mut commands: Commands,
//     scene_query: Query<Entity, With<SceneEntity>>,
//     camera_query: Query<Entity, With<MenuCamera>>,
// ) {
//     println!("进入游戏，清理所有菜单元素");
    
//     // for entity in &scene_query {
//     //     commands.entity(entity).despawn();
//     // }
//     // for entity in &camera_query {
//     //     commands.entity(entity).despawn();
//     // }

// }
fn cleanup_for_game(
    mut commands: Commands,
    scene_query: Query<Entity, With<SceneEntity>>,
    camera_query: Query<Entity, With<MenuCamera>>,
    current_state: Res<State<GameScene>>,
) {
    println!("离开游戏状态，清理所有菜单元素");

    // 检查当前状态是否为 GameSettings
    if *current_state.get() != GameScene::GameSettings {
        // 清理菜单摄像机
        for entity in &camera_query {
            commands.entity(entity).despawn();
        }
    }

    // 隐藏所有菜单UI实体
    for entity in &scene_query {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}


fn cleanup_all_menu(
    mut commands: Commands,
    scene_query: Query<Entity, With<SceneEntity>>,
    camera_query: Query<Entity, With<MenuCamera>>,
    current_state: Res<State<GameScene>>,
) {
    println!("进入游戏，清理所有菜单元素");
    
    // 检查当前状态是否为Game
    if *current_state.get() == GameScene::Game {
        println!("进入游戏状态，清理菜单摄像机");
        // 清理菜单摄像机
        for entity in &camera_query {
            commands.entity(entity).despawn();
        }
    }
    
    // 隐藏所有菜单UI实体
    for entity in &scene_query {
        commands.entity(entity).insert(Visibility::Hidden);
    }
}
fn cleanup_all_about(
    mut commands: Commands,
    about_ui_query: Query<Entity, With<AboutUI>>,
    mut overlay_query: Query<&mut Visibility, With<GameMenuOverlay>>,
) {
    println!("清理关于界面");
    
    // 清理关于界面的UI实体
    for entity in &about_ui_query {
        commands.entity(entity).despawn();
    }
    
    // 隐藏游戏菜单覆盖层
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = Visibility::Hidden;
        println!("已隐藏游戏菜单覆盖层");
    }
}
fn cleanup_load_scene(
    mut commands: Commands,
    load_query: Query<Entity, With<LoadUI>>,
) {
    println!("清理载入界面");
    
    for entity in &load_query {
        commands.entity(entity).despawn();
    }
}
fn cleanup_settings_overlay(
    mut commands: Commands,
    settings_query: Query<Entity, With<SettingsEntity>>,
) {
    println!("清理设置界面");
    
    for entity in &settings_query {
        commands.entity(entity).despawn();
    }
}
fn setup_about_scene(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<MenuCamera>>,
    mut overlay_query: Query<&mut Visibility, With<GameMenuOverlay>>,
    mut main_menu_query: Query<&mut Visibility, (With<MainMenuBackground>, Without<GameMenuOverlay>)>,
) {
    println!("{}","执行关于界面");
    // 确保摄像机存在
    if camera_query.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    // 显示游戏菜单遮罩层
    if let Ok(mut overlay_visibility) = overlay_query.single_mut() {
        *overlay_visibility = Visibility::Visible;
    }

    // // 隐藏主菜单背景（可选，根据你的设计需求）
    // if let Ok(mut main_menu_visibility) = main_menu_query.single_mut() {
    //     *main_menu_visibility = Visibility::Hidden;
    // }

    // 创建关于页面标题（左上角）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(50.0),
            ..default()
        },
        AboutUI, // 用于清理
    )).with_children(|title_parent| {
        title_parent.spawn((
            Text::new("关于"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 45.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });

    // 创建关于页面内容（右侧显示）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(320.0),
            width: Val::Px(500.0),
            height: Val::Px(600.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            // border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
        Visibility::Visible,
        AboutUI, // 用于清理
    )).with_children(|about_parent| {
        // 游戏标题
        about_parent.spawn((
            Text::new("渡鸦引擎"),
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

        // 游戏信息容器
        about_parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(15.0),
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|info_parent| {
            let info_items = [
                ("版本", "0.1.3"),
                ("开发者", "Furau"),
                ("封面画师", "鸮笑笑"),
                ("引擎", "Raven Engine"),
            ];

            for (label, value) in info_items {
                info_parent.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }).with_children(|row_parent| {
                    row_parent.spawn((
                        Text::new(format!("{}:", label)),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                    ));
                    
                    row_parent.spawn((
                        Text::new(value),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }

            // 感谢信息
            info_parent.spawn((
                Text::new("谢谢使用！任何问题可以随时联系Furau@qq.com, 如果您觉得这份程序不错，请去github 点个星星 这对我帮助很大"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });

        // 返回按钮
        about_parent.spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(-220.0),
                width: Val::Px(120.0),
                height: Val::Px(45.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                // border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            // BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
            // BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
            BackButton,
        )).with_children(|button_parent| {
            button_parent.spawn((
                Text::new("返回"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

about_parent.spawn((
    Button,
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(300.0),
        left: Val::Px(10.0),  // 调整位置避免重叠
        width: Val::Px(120.0),
        height: Val::Px(45.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::top(Val::Px(20.0)),
        ..default()
    },
    UrlButton {
        url: "https://github.com/E72UJ/Raven".to_string(),
    },
)).with_children(|button_parent| {
    button_parent.spawn((
        Text::new("GitHub"),
        TextFont {
            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
});
    });
}

fn setup_help_scene(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<MenuCamera>>,
    mut overlay_query: Query<&mut Visibility, With<GameMenuOverlay>>,
    mut main_menu_query: Query<&mut Visibility, (With<MainMenuBackground>, Without<GameMenuOverlay>)>,
) {
    println!("{}","执行帮助界面");
    
    // 确保摄像机存在
    if camera_query.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    // 显示游戏菜单遮罩层
    if let Ok(mut overlay_visibility) = overlay_query.single_mut() {
        *overlay_visibility = Visibility::Visible;
    }

    // 创建帮助页面标题（左上角）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(50.0),
            ..default()
        },
        AboutUI, // 用于清理
    )).with_children(|title_parent| {
        title_parent.spawn((
            Text::new("帮助"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 45.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });

    // 创建帮助页面内容（右侧显示）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(320.0),
            width: Val::Px(500.0),
            height: Val::Px(600.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor::all(Color::srgb(0.5, 0.5, 0.5)),
        Visibility::Visible,
        AboutUI, // 用于清理
    )).with_children(|help_parent| {
        // 游戏标题
        help_parent.spawn((
            Text::new("游戏操作"),
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

        // 操作说明容器
        help_parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(15.0),
            flex_grow: 1.0,
            width: Val::Percent(100.0),
            ..default()
        }).with_children(|info_parent| {
            let help_items = [
                ("回退上一句", "←"),
                ("进入下一句", "Enter"),
                ("退出主界面", "ESC"),
                ("自动播放", "Space"),
            ];

            for (label, key) in help_items {
                info_parent.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                }).with_children(|row_parent| {
                    row_parent.spawn((
                        Text::new(format!("{}:", label)),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                    ));
                    
                    row_parent.spawn((
                        Text::new(key),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }

            // 提示信息
            info_parent.spawn((
                Text::new("提示：游戏支持鼠标点击操作，你也可以使用键盘快捷键来提升游戏体验。按ESC键可以随时返回主菜单。"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });

        // 返回按钮
        help_parent.spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(-220.0),
                width: Val::Px(120.0),
                height: Val::Px(45.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(20.0)),
                ..default()
            },
            BackButton,
        )).with_children(|button_parent| {
            button_parent.spawn((
                Text::new("返回"),
                TextFont {
                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}


fn setup_settings_overlay(mut commands: Commands, asset_server: Res<AssetServer>, camera_query: Query<Entity, With<MenuCamera>>) {
    println!("执行设置界面");
    
    if camera_query.is_empty() {
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
            SettingsEntity,
        ))
        .with_children(|parent| {
            // 设置窗口
            parent
                .spawn((
                    Node {
                        width: Val::Px(650.0),
                        height: Val::Px(580.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.8)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((
                        Text::new("游戏设置"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));

                    // 设置项容器
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: Val::Px(20.0),
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // 音效音量
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("音效音量"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                              // 音量滑块容器
                                    parent
                                        .spawn(Node {
                                            width: Val::Px(200.0),
                                            height: Val::Px(25.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("████████░░"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                                            ));
                                        });
                                });

                            // 背景音乐音量
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("背景音乐音量"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                                    parent
                                        .spawn(Node {
                                            width: Val::Px(200.0),
                                            height: Val::Px(25.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("██████░░░░"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                                            ));
                                        });
                                });

                            // 文字显示速度
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("文字显示速度"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Row,
                                            column_gap: Val::Px(10.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            // 慢速按钮
                                            parent
                                                .spawn((
                                                    Button,
                                                    Node {
                                                        width: Val::Px(50.0),
                                                        height: Val::Px(30.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(Val::Px(1.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                                                    BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
                                                ))
                                                .with_children(|parent| {
                                                    parent.spawn((
                                                        Text::new("慢"),
                                                        TextFont {
                                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                            font_size: 14.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                            
                                            // 中速按钮
                                            parent
                                                .spawn((
                                                    Button,
                                                    Node {
                                                        width:Val::Px(50.0),
                                                        height: Val::Px(30.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(Val::Px(2.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.4, 0.4, 0.6)),
                                                    BorderColor::all(Color::srgb(0.8, 0.8, 1.0)),
                                                ))
                                                .with_children(|parent| {
                                                    parent.spawn((
                                                        Text::new("中"),
                                                        TextFont {
                                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                            font_size: 14.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                            
                                            // 快速按钮
                                            parent
                                                .spawn((
                                                    Button,
                                                    Node {
                                                        width: Val::Px(50.0),
                                                        height: Val::Px(30.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(Val::Px(1.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                                                    BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
                                                ))
                                                .with_children(|parent| {
                                                    parent.spawn((
                                                        Text::new("快"),
                                                        TextFont {
                                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                            font_size: 14.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::WHITE),
                                                    ));
                                                });
                                        });
                                });

                            // 自动播放速度
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("自动播放速度"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                                    parent
                                        .spawn(Node {
                                            width: Val::Px(200.0),
                                            height: Val::Px(25.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("█████░░░░░"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                                            ));
                                        });
                                });

                            // 全屏模式
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("全屏模式"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(80.0),
                                                height: Val::Px(35.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
                                            BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("关闭"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });

                            // 跳过已读文本
                            parent
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("跳过已读文本"),
                                        TextFont {
                                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                            font_size: 18.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                    
                                    parent
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(80.0),
                                                height: Val::Px(35.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.4, 0.4, 0.6)),
                                            BorderColor::all(Color::srgb(0.8, 0.8, 1.0)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("开启"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
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
                            BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
                            BackButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("返回"),
                                TextFont {
                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}
fn setup_load_scene(mut commands: Commands, asset_server: Res<AssetServer>, camera_query: Query<Entity, With<MenuCamera>>) {
    println!("执行载入界面");
    
    if camera_query.is_empty() {
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
            LoadUI,
        ))
        .with_children(|parent| {
            // 载入窗口
            parent
                .spawn((
                    Node {
                        width: Val::Px(700.0),
                        height: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.8)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((
                        Text::new("载入游戏"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 1.0)),
                    ));

                    // 存档槽容器
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: Val::Px(15.0),
                            width: Val::Percent(100.0),
                            height: Val::Px(420.0),
                            overflow: Overflow::clip_y(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // 存档槽1
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::SpaceBetween,
                                        width: Val::Percent(100.0),
                                        height: Val::Px(80.0),
                                        padding: UiRect::all(Val::Px(15.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                                    BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                                ))
                                .with_children(|parent| {
                                    // 存档信息
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Start,
                                            row_gap: Val::Px(5.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("存档槽 1"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                            parent.spawn((
                                                Text::new("第一章 - 你好"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                                            ));
                                            parent.spawn((
                                                Text::new("2025/07/27 14:30"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 12.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                                            ));
                                        });
                                    
                                    // 预览图占位符
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(80.0),
                                            height: Val::Px(50.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
                                        BorderColor::all(Color::srgb(0.6, 0.6, 0.7)),
                                    ));
                                });

                            // 存档槽2
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::SpaceBetween,
                                        width: Val::Percent(100.0),
                                        height: Val::Px(80.0),
                                        padding: UiRect::all(Val::Px(15.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                                    BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                                ))
                                .with_children(|parent| {
                                    parent
                                        .spawn(Node {
                                            flex_direction: FlexDirection::Column,
                                            align_items: AlignItems::Start,
                                            row_gap: Val::Px(5.0),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("存档槽 2"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                            parent.spawn((
                                                Text::new("第二章 - 转折"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 14.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                                            ));
                                            parent.spawn((
                                                Text::new("2025/07/30 20:15"),
                                                TextFont {
                                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                    font_size: 12.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                                            ));
                                        });
                                    
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(80.0),
                                            height: Val::Px(50.0),
                                            border: UiRect::all(Val::Px(1.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
                                        BorderColor::all(Color::srgb(0.6, 0.6, 0.7)),
                                    ));
                                });

                            // 空存档槽
                            for i in 3..=4 {
                                parent
                                    .spawn((
                                        Node {
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::SpaceBetween,
                                            width: Val::Percent(100.0),
                                            height: Val::Px(80.0),
                                            padding: UiRect::all(Val::Px(15.0)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                                        BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                                    ))
                                    .with_children(|parent| {
                                        parent
                                            .spawn(Node {
                                                flex_direction: FlexDirection::Column,
                                                align_items: AlignItems::Start,
                                                row_gap: Val::Px(5.0),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Text::new(&format!("存档槽 {}", i)),
                                                    TextFont {
                                                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                                        font_size: 16.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::srgb(0.6, 0.6, 0.7)),
                                                ));
                                            });
                                        
                                        parent.spawn((
                                            Node {
                                                width: Val::Px(80.0),
                                                height: Val::Px(50.0),
                                                border: UiRect::all(Val::Px(1.0)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
                                            BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                                        ));
                                    });
                            }
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
                            BorderColor::all(Color::srgb(0.5, 0.5, 0.7)),
                            BackButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("返回"),
                                TextFont {
                                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}
// 音频系统
// fn my_system(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // 播放一次性音效
//     // play_audio(&mut commands, &asset_server, "audio/two.ogg");

//     // // 播放音效并设置音量
//     // play_audio_with_volume(&mut commands, &asset_server, "audio/explosion.ogg", 0.7);

//     // // 循环播放背景音乐
//     play_audio_loop(&mut commands, &asset_server, "audio/5gzps-9i0ey.ogg", 1.0);
// }
fn on_exit_game_state(
    mut commands: Commands,
    mut audio_manager: ResMut<AudioManager>,
) {
    // 退出游戏状态时停止所有音频
    println!("{}","推出所有饮品");
    stop_all_audio(&mut commands, &mut audio_manager);
}

// 放大系统
fn update_background_size_on_resize(
    mut resize_events: MessageReader<WindowResized>,
    mut sprite_query: Query<&mut Sprite>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // 只在窗口大小改变时执行
    for _event in resize_events.read() {
        if let Ok(window) = window_query.single() {
            for mut sprite in sprite_query.iter_mut() {
                let image_aspect = 2560.0 / 1440.0; // 原图比例
                let window_aspect = window.width() / window.height(); // 窗口比例
                
                let (new_width, new_height) = if window_aspect > image_aspect {
                    // 窗口更宽，以高度为准缩放
                    let height = window.height();
                    let width = height * image_aspect;
                    (width, height)
                } else {
                    // 窗口更窄，以宽度为准缩放
                    let width = window.width();
                    let height = width / image_aspect;
                    (width, height)
                };
                
                sprite.custom_size = Some(Vec2::new(new_width, new_height));
            }
        }
    }
}

fn cleanup_cameras(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>,
) {
    println!("相机清理函数");
    for camera_entity in cameras.iter() {
        commands.entity(camera_entity).despawn();
    }
}


fn setup_game_settings_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_query: Query<Entity, With<MenuCamera>>,
    mut overlay_query: Query<&mut Visibility, With<GameMenuOverlay>>,
) {
    println!("游戏设置界面");
    
    // 确保摄像机存在
    if camera_query.is_empty() {
        commands.spawn((Camera2d, MenuCamera));
    }

    // 显示游戏菜单遮罩层
    if let Ok(mut overlay_visibility) = overlay_query.single_mut() {
        *overlay_visibility = Visibility::Visible;
    }

    // 创建设置页面标题（左上角）
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(50.0),
            ..default()
        },
        SettingsEntity, // 用于清理
    )).with_children(|title_parent| {
        title_parent.spawn((
            Text::new("设置"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Light.ttf"),
                font_size: 45.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.6, 0.2)), // 橙色
        ));
    });

    // 创建主设置界面容器
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        SettingsEntity,
    )).with_children(|main_parent| {
        // 左侧菜单栏
        main_parent.spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Px(50.0),
                    top: Val::Px(120.0),
                    right: Val::Px(20.0),
                    bottom: Val::Px(50.0),
                },
                row_gap: Val::Px(20.0),
                ..default()
            },
        )).with_children(|menu_parent| {
            let menu_items = [
                "历史",
                "保存",
                "读取游戏",
                "设置",
                "标题界面", 
                "关于",
                "帮助",
                "返出"
            ];

            for (index, item) in menu_items.iter().enumerate() {
                let is_selected = index == 3; // "设置" 被选中
                
                menu_parent.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::left(Val::Px(10.0)),
                        border: if is_selected { 
                            UiRect::left(Val::Px(3.0)) 
                        } else { 
                            UiRect::ZERO 
                        },
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                    BorderColor::all(Color::srgb(1.0, 0.6, 0.2)), // 橙色边框
                )).with_children(|button_parent| {
                    button_parent.spawn((
                        Text::new(*item),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: if is_selected { 20.0 } else { 18.0 },
                            ..default()
                        },
                        TextColor(if is_selected { 
                            Color::srgb(1.0, 0.6, 0.2) // 橙色文字
                        } else { 
                            Color::WHITE 
                        }),
                    ));
                });
            }
        });

        // 右侧设置内容区域
        main_parent.spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect {
                    left: Val::Px(50.0),
                    top: Val::Px(120.0),
                    right: Val::Px(100.0),
                    bottom: Val::Px(50.0),
                },
                row_gap: Val::Px(50.0),
                ..default()
            },
        )).with_children(|content_parent| {
            // 显示设置区域
            content_parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(30.0),
                    ..default()
                },
            )).with_children(|settings_parent| {
                // 显示设置组
                settings_parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                )).with_children(|display_parent| {
                    // 显示标题
                    display_parent.spawn((
                        Text::new("显示"),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.2)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 窗口/全屏设置
                    display_parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(30.0),
                            ..default()
                        },
                    )).with_children(|row_parent| {
                        // 窗口按钮
                        row_parent.spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                            BorderColor::all(Color::srgb(1.0, 0.6, 0.2)),
                        )).with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("窗口"),
                                TextFont {
                                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.6, 0.2)),
                            ));
                        });

                        // 全屏按钮
                        row_parent.spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                            BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                        )).with_children(|button_parent| {
                            button_parent.spawn((
                                Text::new("全屏"),
                                TextFont {
                                    font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
                });

                // 语言设置组
                settings_parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(20.0),
                        ..default()
                    },
                )).with_children(|language_parent| {
                    // 语言标题
                    language_parent.spawn((
                        Text::new("语言"),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.2)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 语言选择网格
                    language_parent.spawn((
                        Node {
                            display: Display::Grid,
                            grid_template_columns: vec![
                                RepeatedGridTrack::auto(2)
                            ],
                            column_gap: Val::Px(40.0),
                            row_gap: Val::Px(20.0),
                            width: Val::Percent(80.0),
                            ..default()
                        },
                    )).with_children(|grid_parent| {
                        let languages = [
                            ("English", false),
                            ("Español", false),
                            ("Česky", false),
                            ("Українська", false),
                            ("Dansk", false),
                            ("日本語", false),
                            ("Français", false),
                            ("한국어", false),
                            ("Italiano", false),
                            ("简体中文", true), // 选中状态
                            ("Bahasa Melayu", false),
                            ("繁體中文", false),
                            ("Русский", false),
                        ];

                        for (lang, is_selected) in languages {
                            grid_parent.spawn((
                                Button,
                                Node {
                                    padding: UiRect::all(Val::Px(10.0)),
                                    border: if is_selected { 
                                        UiRect::all(Val::Px(2.0)) 
                                    } else { 
                                        UiRect::ZERO 
                                    },
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                BorderColor::all(Color::srgb(1.0, 0.6, 0.2)),
                            )).with_children(|lang_parent| {
                                lang_parent.spawn((
                                    Text::new(lang),
                                    TextFont {
                                        font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(if is_selected { 
                                        Color::srgb(1.0, 0.6, 0.2) 
                                    } else { 
                                        Color::WHITE 
                                    }),
                                ));
                            });
                        }
                    });
                });
            });
        });
    });

    // 返回按钮（左下角）
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            left: Val::Px(50.0),
            width: Val::Px(120.0),
            height: Val::Px(45.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        BackButton,
        SettingsEntity,
    )).with_children(|button_parent| {
        button_parent.spawn((
            Text::new("返回"),
            TextFont {
                font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                font_size: 30.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn cleanup_game_settings_overlay(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>,
){
    println!("游戏设置界面结束");
}

