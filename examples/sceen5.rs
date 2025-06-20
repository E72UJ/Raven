use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    SplashScreen,
    MainMenu,
    InGame,
}

#[derive(Resource)]
struct SplashTimer(Timer);

#[derive(Component)]
struct SplashScreenEntity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: Vec2::new(800.0, 600.0).into(),
                title: "开屏动画示例".to_string(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::SplashScreen), load_splash_screen)
        .add_systems(Update, splash_screen_animation.run_if(in_state(AppState::SplashScreen)))
        .add_systems(OnExit(AppState::SplashScreen), cleanup_splash_screen)
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_splash_screen(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // 插入计时器资源
    commands.insert_resource(SplashTimer(Timer::from_seconds(3.0, TimerMode::Once)));
    
    // 创建简单的开屏动画 - 使用精灵
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.4, 0.8),
            custom_size: Some(Vec2::new(800.0, 600.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        SplashScreenEntity,
    ));
    
    // 添加一个白色的中心方块作为动画元素
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        SplashScreenEntity,
    ));
}

fn splash_screen_animation(
    time: Res<Time>,
    mut splash_timer: ResMut<SplashTimer>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<&mut Transform, With<SplashScreenEntity>>,
) {
    splash_timer.0.tick(time.delta());
    
    // 简单的旋转动画
    for mut transform in query.iter_mut() {
        if transform.translation.z > 0.5 { // 只旋转白色方块
            transform.rotate_z(time.delta_secs() * 2.0);
        }
    }
    
    if splash_timer.0.finished() {
        next_state.set(AppState::MainMenu);
    }
}

fn cleanup_splash_screen(
    mut commands: Commands,
    query: Query<Entity, With<SplashScreenEntity>>,
) {
    // 移除计时器资源
    commands.remove_resource::<SplashTimer>();
    
    // 销毁开屏动画实体
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 创建简单的主菜单UI
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new("主菜单"),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}