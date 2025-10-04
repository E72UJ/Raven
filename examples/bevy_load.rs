use bevy::prelude::*;

#[derive(Component)]
pub struct Background;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("background_one"),
        Background,
        Sprite {
            image: asset_server.load("background/one.png"),
            // 通过设置 custom_size 来减少渲染尺寸，从而减少内存占用
            custom_size: Some(Vec2::new(800.0, 600.0)), // 根据需要调整尺寸
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -10.0),
    ));
    
    println!("已加载背景: background/one.png");
}

// 可选：当不需要时卸载资源
fn cleanup_system(
    mut commands: Commands,
    query: Query<Entity, With<Background>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, load_background))
        // .add_systems(Update, cleanup_system) // 需要时启用
        .run();
}
