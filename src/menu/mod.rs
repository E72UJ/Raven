// src/menu/mod.rs
use bevy::{input_focus::InputFocus, prelude::*, winit::WinitSettings};
use crate::GameScene;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // 只有在有用户输入时才运行应用程序，这将显著减少CPU/GPU使用
            .insert_resource(WinitSettings::desktop_app())
            // 必须设置 `InputFocus` 以便辅助功能识别按钮
            .init_resource::<InputFocus>()
            .init_state::<GameScene>()
            .add_systems(Startup, setup)
            .add_systems(Update, button_system)
            .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
            .add_systems(OnEnter(GameScene::Menu), setup_menu_scene)
            .add_systems(OnEnter(GameScene::Game), setup_game_scene)
            .add_systems(OnEnter(GameScene::Settings), setup_settings_scene)
            .add_systems(OnExit(GameScene::Menu), cleanup_scene)
            .add_systems(OnExit(GameScene::Game), cleanup_scene)
            .add_systems(OnExit(GameScene::Settings), cleanup_scene);
    }
}

// 组件定义
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
struct MenuToGameButton;

#[derive(Component)]
struct GameToSettingsButton;

#[derive(Component)]
struct SettingsToMenuButton;

// 按钮颜色常量
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.1, 0.1, 0.1);

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
            Option<&MenuToGameButton>,
            Option<&GameToSettingsButton>,
            Option<&SettingsToMenuButton>,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children, menu_to_game, game_to_settings, settings_to_menu) in
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
                if menu_to_game.is_some() {
                    next_state.set(GameScene::Game);
                } else if game_to_settings.is_some() {
                    next_state.set(GameScene::Settings);
                } else if settings_to_menu.is_some() {
                    next_state.set(GameScene::Menu);
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
    commands.spawn(Camera2d);
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
                            margin: UiRect::bottom(Val::Px(50.0)),
                            ..default()
                        },
                    ),
                    create_button(&assets, "开始游戏", MenuToGameButton),
                    create_button(&assets, "设置", SettingsButton),
                    create_button(&assets, "载入存档", LoadGameButton),
                ],
            ),
            // 右侧图片区域
            (
                Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    ImageNode::new(assets.load("background/bg2.png")),
                    Node {
                        width: Val::Px(1152.0),
                        height: Val::Px(800.0),
                        ..default()
                    },
                )],
            ),
        ],
    ));
}

fn setup_game_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new("Game Scene"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                Text::new("This is where your game would be!"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ),
        ],
    ));
}

fn setup_settings_scene(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        SceneEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        children![
            (
                Text::new("Settings"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ),
            (
                Text::new("Configure your game settings here"),
                TextFont {
                    font: assets.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ),
            create_button(&assets, "Back to Menu", SettingsToMenuButton)
        ],
    ));
}

fn create_button(asset_server: &AssetServer, text: &str, button_type: impl Component) -> impl Bundle {
    (
        button_type,
        Button,
        Node {
            width: Val::Px(290.0),
            height: Val::Px(65.0),
            border: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        BackgroundColor(NORMAL_BUTTON),
        children![(
            Text::new(text),
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )]
    )
}

fn cleanup_scene(mut commands: Commands, query: Query<Entity, With<SceneEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}