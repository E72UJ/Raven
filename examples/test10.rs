use bevy::{asset::LoadState, prelude::*};
use std::fmt;

// 定义场景状态
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameScene {
    #[default]
    A,
    B,
}

// 为场景状态实现Display trait以便于日志输出
impl fmt::Display for GameScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameScene::A => write!(f, "Scene A"),
            GameScene::B => write!(f, "Scene B"),
        }
    }
}

// 场景标记组件
#[derive(Component)]
struct SceneAEntity;

#[derive(Component)]
struct SceneBEntity;

// 切换场景按钮组件
#[derive(Component)]
struct SwitchSceneButton;

// 场景资源，用于存储场景特定的数据
#[derive(Resource)]
struct SceneData {
    background_image: Handle<Image>,
}

// 主插件
pub struct SceneSwitcherPlugin;

impl Plugin for SceneSwitcherPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScene>()
            .add_systems(OnEnter(GameScene::A), setup_scene_a)
            .add_systems(OnExit(GameScene::A), cleanup_scene_a)
            .add_systems(OnEnter(GameScene::B), setup_scene_b)
            .add_systems(OnExit(GameScene::B), cleanup_scene_b)
            .add_systems(Update, (switch_scene_system, check_scene_loaded));
    }
}

// 设置场景A
fn setup_scene_a(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setting up Scene A");

    // 创建场景特定的资源
    commands.insert_resource(SceneData {
        background_image: asset_server.load("gui/game3.png"),
    });

    // 创建相机
    commands.spawn((Camera2d, SceneAEntity));

    // 创建UI
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
            SceneAEntity,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("Scene A"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 切换场景按钮
            parent
                .spawn((
                    Button,
                    SwitchSceneButton,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    SceneAEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Go to Scene B"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });

    // 创建背景图片
    commands.spawn((
        ImageNode::new(asset_server.load("gui/game3.png")),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(-1), // 确保背景在UI后面
        SceneAEntity,
    ));
}

// 清理场景A
fn cleanup_scene_a(mut commands: Commands, query: Query<Entity, With<SceneAEntity>>) {
    info!("Cleaning up Scene A");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // 移除场景特定的资源
    commands.remove_resource::<SceneData>();
}

// 设置场景B
fn setup_scene_b(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setting up Scene B");

    // 创建场景特定的资源
    commands.insert_resource(SceneData {
        background_image: asset_server.load("gui/textbox.png"),
    });

    // 创建相机
    commands.spawn((Camera2d, SceneBEntity));

    // 创建UI
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
            SceneBEntity,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("Scene B"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 切换场景按钮
            parent
                .spawn((
                    Button,
                    SwitchSceneButton,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    SceneBEntity,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Go to Scene A"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });

    // 创建背景图片
    commands.spawn((
        ImageNode::new(asset_server.load("gui/textbox.png")),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(-1), // 确保背景在UI后面
        SceneBEntity,
    ));
}

// 清理场景B
fn cleanup_scene_b(mut commands: Commands, query: Query<Entity, With<SceneBEntity>>) {
    info!("Cleaning up Scene B");

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // 移除场景特定的资源
    commands.remove_resource::<SceneData>();
}

// 切换场景系统
fn switch_scene_system(
    mut next_state: ResMut<NextState<GameScene>>,
    interaction_query: Query<
        &Interaction,
        (With<Button>, With<SwitchSceneButton>, Changed<Interaction>),
    >,
    current_state: Res<State<GameScene>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let next_scene = match current_state.get() {
                GameScene::A => GameScene::B,
                GameScene::B => GameScene::A,
            };

            info!("Switching from {} to {}", current_state.get(), next_scene);
            next_state.set(next_scene);
        }
    }
}

// 检查场景资源是否加载完成
fn check_scene_loaded(
    asset_server: Res<AssetServer>,
    scene_data: Option<Res<SceneData>>,
    current_state: Res<State<GameScene>>,
) {
    // println!("Checking scene loaded for {}", current_state.get());
}

// 主函数
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SceneSwitcherPlugin)
        .run();
}
