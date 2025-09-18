use bevy::prelude::*;
use bevy::window::{Window, PrimaryWindow};
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .insert_resource(ClearColor(Color::BLACK))  // 设置为纯黑色
        .add_systems(Startup, (setup_background,setup_background2))
        .add_systems(Update, update_background_size)
        .run();
}


fn update_background_size(
    mut sprite_query: Query<&mut Sprite>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.get_single() {
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

fn setup_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = window_query.get_single() {
        let aspect_ratio = 2560.0 / 1440.0;
        let scaled_height = window.width() / aspect_ratio;
        println!("Scaled height: {}", aspect_ratio);
        commands.spawn((
            Sprite {
                image: asset_server.load("gui/main_menu.png"),
                custom_size: Some(Vec2::new(window.width(), scaled_height)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.2)),
        ));
    }
}

fn setup_background2(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    commands.spawn(Camera2d);
    if let Ok(window) = window_query.get_single() {
        let aspect_ratio = 2560.0 / 1440.0;
        let scaled_height = window.width() / aspect_ratio;
        commands.spawn((
            Sprite {
                image: asset_server.load("gui/game3.png"),
                custom_size: Some(Vec2::new(window.width(), scaled_height)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ));
            }
}