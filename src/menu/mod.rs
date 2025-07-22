// src/menu/mod.rs
use bevy::{input_focus::InputFocus, prelude::*, winit::WinitSettings};
use crate::GameScene;

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
            .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
            // 只管理菜单场景
            .add_systems(OnEnter(GameScene::Menu), setup_menu_scene)
            .add_systems(OnExit(GameScene::Menu), cleanup_all_menu)
            .add_systems(OnEnter(GameScene::Settings), setup_settings_overlay)
            .add_systems(OnExit(GameScene::Settings), cleanup_scene);


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
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children, start_game, settings, exit_game) in
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
                } else if exit_game.is_some() {
                    // 处理退出游戏
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

fn setup_settings_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("在主菜单基础上添加设置界面...");
    
    // 创建半透明遮罩背景
    commands.spawn((
        Sprite {
            color: Color::srgba(0.2, 0.4, 0.6, 0.7),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
        SettingsEntity,
    ));
    
    // 设置面板背景
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.3),
            custom_size: Some(Vec2::new(500.0, 400.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 6.0)),
        SettingsEntity,
    ));
    
    // 标题文字
    commands.spawn((
        Text2d::new("设置"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 150.0, 7.0)),
        SettingsEntity,
    ));
    
    // 音量设置文字
    commands.spawn((
        Text2d::new("音量"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(-150.0, 50.0, 7.0)),
        SettingsEntity,
    ));
    
    // 音量滑块背景
    commands.spawn((
        Sprite {
            color: Color::srgb(0.4, 0.4, 0.4),
            custom_size: Some(Vec2::new(200.0, 10.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(50.0, 50.0, 7.0)),
        SettingsEntity,
    ));
    
    // 音量滑块
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(20.0, 50.0, 7.5)),
        SettingsEntity,
    ));
    
    // 返回按钮
    commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.3, 0.3),
            custom_size: Some(Vec2::new(120.0, 40.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -120.0, 7.0)),
        SettingsEntity,
    ));
    
    // 返回按钮文字
    commands.spawn((
        Text2d::new("返回"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -120.0, 7.5)),
        SettingsEntity,
    ));
}

