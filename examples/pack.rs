use bevy::prelude::*;
use bevy::input::mouse::MouseButtonInput;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_mouse_movement,
            handle_mouse_click,
            update_hover_visual,
            draw_contour_borders,
        ))
        .run();
}

// 组件定义
#[derive(Component)]
struct IrregularClickArea {
    vertices: Vec<Vec2>,
}

#[derive(Component)]
struct Hovered(bool);

#[derive(Component)]
struct LampEntity;

#[derive(Component)]
struct BottomDecorationEntity;

#[derive(Component)]
struct ShowBorder(bool);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    
    // 台灯实体
    commands.spawn((
        Sprite {
            image: asset_server.load("images/1.png"),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        IrregularClickArea {
            vertices: vec![
                Vec2::new(-40.0, 60.0),
                Vec2::new(-45.0, 45.0),
                Vec2::new(-35.0, 30.0),
                Vec2::new(-8.0, 25.0),
                Vec2::new(-8.0, -30.0),
                Vec2::new(-25.0, -35.0),
                Vec2::new(-30.0, -50.0),
                Vec2::new(30.0, -50.0),
                Vec2::new(25.0, -35.0),
                Vec2::new(8.0, -30.0),
                Vec2::new(8.0, 25.0),
                Vec2::new(35.0, 30.0),
                Vec2::new(45.0, 45.0),
                Vec2::new(40.0, 60.0),
            ],
        },
        Hovered(false),
        ShowBorder(true),
        LampEntity,
    ));
    
    // 底部装饰图片实体
    commands.spawn((
        Sprite {
            image: asset_server.load("images/6.png"),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -120.0, 0.0)),
        IrregularClickArea {
            vertices: vec![
                Vec2::new(-50.0, 25.0),
                Vec2::new(50.0, 25.0),
                Vec2::new(50.0, -25.0),
                Vec2::new(-50.0, -25.0),
            ],
        },
        Hovered(false),
        ShowBorder(true),
        BottomDecorationEntity,
    ));
}

fn handle_mouse_movement(
    mut query: Query<(&Transform, &IrregularClickArea, &mut Hovered)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = q_window.single() else { return };
    let Ok((camera, camera_transform)) = q_camera.single() else { return };
    
    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (transform, click_area, mut hovered) in query.iter_mut() {
                // 将世界坐标转换为相对于实体的本地坐标
                let local_position = world_position - transform.translation.xy();
                
                // 检查点是否在多边形内
                let is_inside = point_in_polygon(local_position, &click_area.vertices);
                hovered.0 = is_inside;
            }
        }
    } else {
        // 鼠标不在窗口内,取消所有悬停状态
        for (_, _, mut hovered) in query.iter_mut() {
            hovered.0 = false;
        }
    }
}

fn handle_mouse_click(
    mut mouse_button_input: EventReader<MouseButtonInput>,
    query: Query<(&Hovered, Option<&LampEntity>, Option<&BottomDecorationEntity>)>,
) {
    for event in mouse_button_input.read() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            for (hovered, lamp, decoration) in query.iter() {
                if hovered.0 {
                    if lamp.is_some() {
                        println!("台灯被点击了!");
                        // 在这里添加台灯的点击逻辑
                    } else if decoration.is_some() {
                        println!("底部装饰被点击了!");
                        // 在这里添加底部装饰的点击逻辑
                    }
                }
            }
        }
    }
}

fn update_hover_visual(
    mut query: Query<(&mut Sprite, &Hovered, Option<&LampEntity>, Option<&BottomDecorationEntity>)>,
) {
    for (mut sprite, hovered, lamp, decoration) in query.iter_mut() {
        if hovered.0 {
            if lamp.is_some() {
                // 台灯悬停效果:变亮
                sprite.color = Color::srgb(1.2, 1.2, 1.2);
            } else if decoration.is_some() {
                // 底部装饰悬停效果:变成淡蓝色
                sprite.color = Color::srgb(0.8, 0.8, 1.2);
            }
        } else {
            // 恢复原色
            sprite.color = Color::WHITE;
        }
    }
}

fn draw_contour_borders(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &IrregularClickArea, &ShowBorder, &Hovered)>,
) {
    for (transform, click_area, show_border, hovered) in query.iter() {
        if !show_border.0 {
            continue;
        }
        
        // 根据悬停状态选择边框颜色
        let border_color = if hovered.0 {
            Color::srgb(1.0, 0.5, 0.0) // 悬停时橙色
        } else {
            Color::srgb(0.0, 1.0, 0.0) // 默认绿色
        };
        
        let vertices = &click_area.vertices;
        if vertices.len() < 3 {
            continue;
        }
        
        // 绘制边框线段
        for i in 0..vertices.len() {
            let current = vertices[i];
            let next = vertices[(i + 1) % vertices.len()];
            
            // 转换为世界坐标
            let world_current = transform.translation.xy() + current;
            let world_next = transform.translation.xy() + next;
            
            gizmos.line_2d(world_current, world_next, border_color);
        }
    }
}

// 点在多边形内检测算法(Ray Casting)
fn point_in_polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    let x = point.x;
    let y = point.y;
    let n = vertices.len();
    let mut inside = false;

    let mut p1x = vertices[0].x;
    let mut p1y = vertices[0].y;

    for i in 1..=n {
        let p2x = vertices[i % n].x;
        let p2y = vertices[i % n].y;

        if y > p1y.min(p2y) {
            if y <= p1y.max(p2y) {
                if x <= p1x.max(p2x) {
                    if p1y != p2y {
                        let xinters = (y - p1y) * (p2x - p1x) / (p2y - p1y) + p1x;
                        if p1x == p2x || x <= xinters {
                            inside = !inside;
                        }
                    }
                }
            }
        }
        p1x = p2x;
        p1y = p2y;
    }

    inside
}
