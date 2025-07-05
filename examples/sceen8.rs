use bevy::prelude::*;
use bevy::window::WindowResized;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, resize_sprite_system)
        .run();
}

#[derive(Component)]
struct BackgroundSprite;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Bevy 0.16中使用Camera2d
    commands.spawn(Camera2d);

    // 背景图片 - 手动添加各个组件
    commands.spawn((
        Sprite {
            image: asset_server.load("background/bg2.png"),
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            ..default()
        },
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        BackgroundSprite,
    ));
}

fn resize_sprite_system(
    mut resize_events: EventReader<WindowResized>,
    mut sprite_query: Query<&mut Sprite, With<BackgroundSprite>>,
) {
    for event in resize_events.read() {
        for mut sprite in sprite_query.iter_mut() {
            sprite.custom_size = Some(Vec2::new(event.width, event.height));
        }
    }
}