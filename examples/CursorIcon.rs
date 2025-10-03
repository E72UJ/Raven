use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy::winit::cursor::CursorIcon;

#[derive(Component)]
struct HoverShape {
    cursor_style: CursorIcon,
    name: String,
}

// 弹窗组件
#[derive(Component)]
pub struct Popup {
    pub timer: Timer,
}

impl Popup {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

// 弹窗生成函数
pub fn spawn_popup(
    commands: &mut Commands,
    message: String,
    position: Vec2,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(position.x - 100.0), // 居中显示
            top: Val::Px(position.y - 50.0),
            width: Val::Px(200.0),
            height: Val::Px(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        BorderColor(Color::WHITE),
        BorderRadius::all(Val::Px(10.0)),
        Popup::new(1.5), // 1.5秒后消失
        ZIndex(1000), // 确保在最顶层
    )).with_children(|parent| {
        parent.spawn((
            Text::new(message),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "交互案例".to_string(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            cursor_hover_system, 
            mouse_click_system,
            update_popups, // 添加弹窗更新系统
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>, 
) {
    // 摄像机
    commands.spawn(Camera2d);

    // 正方形
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(1.0, 0.0, 0.0)))),
        Sprite {
            image: asset_server.load("images/1.png"),
            custom_size: Some(Vec2::new(1440.0, 750.0)), 
            ..default()
        },
        Transform::from_translation(Vec3::new(-200.0, 0.0, 0.0)),
        HoverShape {
            cursor_style: SystemCursorIcon::Crosshair.into(),
            name: "红色正方形".to_string(),
        },
    ));

    // 三角形
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(150.0, 3))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 1.0, 0.0)))),
        Transform::from_translation(Vec3::new(0.0, -200.0, 1.0)),
        HoverShape {
            cursor_style: SystemCursorIcon::Pointer.into(),
            name: "绿色三角形".to_string(),
        },
    ));

    // 圆形
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 0.0, 1.0)))),
        Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
    ));
}

fn cursor_hover_system(
    mut commands: Commands,
    window_query: Query<Entity, With<Window>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    shape_query: Query<(&Transform, &HoverShape)>,
    windows: Query<&Window>,
) {
    let Ok(window_entity) = window_query.single() else { return; };
    let Ok(window) = windows.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        let mut cursor_changed = false;

        for (transform, hover_shape) in shape_query.iter() {
            let shape_pos = transform.translation.truncate();
            let distance = world_position.distance(shape_pos);

            if distance < 70.0 {
                commands.entity(window_entity).insert(hover_shape.cursor_style.clone());
                cursor_changed = true;
                break;
            }
        }

        if !cursor_changed {
            commands.entity(window_entity).insert(CursorIcon::from(SystemCursorIcon::Default));
        }
    }
}

// 修改后的鼠标点击检测系统，集成弹窗功能
fn mouse_click_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    shape_query: Query<(&Transform, &HoverShape)>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return; };
        let Ok((camera, camera_transform)) = camera_query.single() else { return; };

        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            // 获取屏幕坐标用于弹窗定位
            if let Some(screen_position) = window.cursor_position() {
                for (transform, hover_shape) in shape_query.iter() {
                    let shape_pos = transform.translation.truncate();
                    let distance = world_position.distance(shape_pos);

                    if distance < 70.0 {
                        println!("点击了 {} 在位置 ({:.1}, {:.1})", 
                            hover_shape.name, 
                            world_position.x, 
                            world_position.y
                        );
                        
                        // 显示弹窗
                        spawn_popup(
                            &mut commands,
                            format!("computer: {}","this is computer"),
                            screen_position,
                        );
                        break;
                    }
                }
            }
        }
    }
}

// 弹窗更新和清理系统
fn update_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut popup_query: Query<(Entity, &mut Popup)>,
) {
    for (entity, mut popup) in popup_query.iter_mut() {
        popup.timer.tick(time.delta());
        
        if popup.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
