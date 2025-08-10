use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy::winit::cursor::CursorIcon;

#[derive(Component)]
struct HoverShape {
    cursor_style: CursorIcon,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "简单光标示例".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, cursor_hover_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 摄像机
    commands.spawn(Camera2d);

    // 正方形
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
        Transform::from_translation(Vec3::new(-200.0, 0.0, 0.0)),
        HoverShape {
            cursor_style: SystemCursorIcon::Pointer.into(),
        },
    ));

    // 三角形
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(60.0, 3))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 1.0, 0.0)))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        HoverShape {
            cursor_style: SystemCursorIcon::Text.into(),
        },
    ));

    // 圆形
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 0.0, 1.0)))),
        Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
        HoverShape {
            cursor_style: SystemCursorIcon::Move.into(),
        },
    ));
}

fn cursor_hover_system(
    mut commands: Commands,
    window_query: Query<Entity, With<Window>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    shape_query: Query<(&Transform, &HoverShape)>,
    windows: Query<&Window>,
) {
    let Ok(window_entity) = window_query.get_single() else { return; };
    let Ok(window) = windows.get_single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };

    // 获取鼠标在世界坐标中的位置
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let mut cursor_changed = false;

        // 检查鼠标是否悬停在任何形状上
        for (transform, hover_shape) in shape_query.iter() {
            let shape_pos = transform.translation.truncate();
            let distance = world_position.distance(shape_pos);

            // 简单的距离检测
            if distance < 70.0 {
                commands.entity(window_entity).insert(hover_shape.cursor_style.clone());
                cursor_changed = true;
                break;
            }
        }

        // 如果没有悬停在任何形状上，恢复默认光标
        if !cursor_changed {
            commands.entity(window_entity).insert(CursorIcon::from(SystemCursorIcon::Default));
        }
    }
}
