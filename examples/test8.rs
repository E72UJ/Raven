use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_sprite_clicks)
        .run();
}

#[derive(Component)]
struct ClickableSprite {
    message: String,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    // 可点击的 Sprite
    commands.spawn((
        Sprite::from_image(asset_server.load("fps/4.png")),
        Transform::from_xyz(-200.0, 0.0, 0.0),
        ClickableSprite {
            message: "点击了 Sprite！".to_string(),
        },
    ));

    // 彩色方块
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        Transform::from_xyz(200.0, 0.0, 0.0),
        ClickableSprite {
            message: "点击了红色方块！".to_string(),
        },
    ));
}

fn handle_sprite_clicks(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    sprites: Query<(&Transform, &ClickableSprite), With<Sprite>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_pos) = window.cursor_position() {
                for (camera, camera_transform) in &cameras {
                    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        for (sprite_transform, clickable) in &sprites {
                            let sprite_pos = sprite_transform.translation.truncate();
                            let distance = world_pos.distance(sprite_pos);
                            
                            // 假设 Sprite 大小为 50x50 像素
                            if distance < 500.0 {
                                println!("{}", clickable.message);
                            }
                        }
                    }
                }
            }
        }
    }
}
