use bevy::prelude::*;

#[derive(Component)]
struct HoverArea {
    rect: Rect,
    id: String,
}

#[derive(Component)]
struct BorderOutline {
    target_entity: Entity,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_hover_areas)
        .add_systems(Update, hover_area_system)
        .run();
}

fn setup_hover_areas(mut commands: Commands) {
    commands.spawn(Camera2d);

    // 创建透明的可悬停区域
    let hover_entity = commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0), // 完全透明
            custom_size: Some(Vec2::new(200.0, 150.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        HoverArea {
            rect: Rect::from_center_size(Vec2::ZERO, Vec2::new(200.0, 150.0)),
            id: "hover_area_1".to_string(),
        },
    )).id();

    // 创建4个边框线条（上下左右）
    let border_thickness = 2.0;
    let area_size = Vec2::new(200.0, 150.0);

    // 上边框
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(area_size.x + border_thickness * 2.0, border_thickness)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, area_size.y / 2.0 + border_thickness / 2.0, 0.1)),
        Visibility::Hidden,
        BorderOutline { target_entity: hover_entity },
    ));

    // 下边框
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(area_size.x + border_thickness * 2.0, border_thickness)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -area_size.y / 2.0 - border_thickness / 2.0, 0.1)),
        Visibility::Hidden,
        BorderOutline { target_entity: hover_entity },
    ));

    // 左边框
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(border_thickness, area_size.y)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-area_size.x / 2.0 - border_thickness / 2.0, 0.0, 0.1)),
        Visibility::Hidden,
        BorderOutline { target_entity: hover_entity },
    ));

    // 右边框
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(border_thickness, area_size.y)),
            ..default()
        },
        Transform::from_translation(Vec3::new(area_size.x / 2.0 + border_thickness / 2.0, 0.0, 0.1)),
        Visibility::Hidden,
        BorderOutline { target_entity: hover_entity },
    ));
}

fn hover_area_system(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    hover_query: Query<(Entity, &Transform, &HoverArea)>,
    mut border_query: Query<(&mut Visibility, &mut Transform, &BorderOutline), Without<HoverArea>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    let Ok(window) = windows.get_single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return; };
    
    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            let mut hovered_entities = std::collections::HashSet::new();
            
            // 检查哪些区域被悬停
            for (entity, transform, hover_area) in hover_query.iter() {
                let world_rect = Rect::from_center_size(
                    transform.translation.truncate(),
                    hover_area.rect.size()
                );
                
                if world_rect.contains(world_position) {
                    hovered_entities.insert(entity);
                    
                    // 处理点击
                    if mouse_button_input.just_pressed(MouseButton::Left) {
                        println!("点击了区域: {}", hover_area.id);
                        // 在这里执行你想要的代码
                    }
                }
            }
            
            // 更新边框显示状态
            for (mut visibility, mut border_transform, border_outline) in border_query.iter_mut() {
                if hovered_entities.contains(&border_outline.target_entity) {
                    *visibility = Visibility::Visible;
                    
                    // 更新边框位置以跟随目标实体
                    if let Ok((_, target_transform, _)) = hover_query.get(border_outline.target_entity) {
                        // 保持边框的相对偏移，但跟随目标实体的位置
                        let offset = border_transform.translation - Vec3::ZERO;
                        border_transform.translation = target_transform.translation + offset;
                    }
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    } else {
        // 鼠标不在窗口内，隐藏所有边框
        for (mut visibility, _, _) in border_query.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}
