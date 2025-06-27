use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_image_animation)
        .run();
}

#[derive(Resource)]
struct AnimatedImage {
    entity: Entity,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 创建相机
    commands.spawn(Camera2d);

    // 创建带动画的ImageNode
    let image_entity = commands
        .spawn((
            Name::new("animated_background"),
            ImageNode {
                image: asset_server.load("background/42B7A693CCADB04C9370D5E36FA40C45.png"),
                flip_x: true,
                flip_y: false,
                ..default()
            },
            Node {
                width: Val::Px(200.0),
                position_type: PositionType::Absolute,
                left: Val::Px(100.0),
                top: Val::Px(100.0),
                ..default()
            },
            Visibility::Visible,
        ))
        .id();

    // 存储图片实体用于动画
    commands.insert_resource(AnimatedImage { entity: image_entity });
}

fn update_image_animation(
    time: Res<Time>,
    animated_image: Res<AnimatedImage>,
    mut query: Query<&mut Node>,
) {
    if let Ok(mut node) = query.get_mut(animated_image.entity) {
        // 水平晃动
        let horizontal_offset = (time.elapsed_secs() * 2.0).sin() * 10.0;
        node.left = Val::Px(100.0 + horizontal_offset);
        
        // 垂直晃动（可选）
        let vertical_offset = (time.elapsed_secs() * 4.0).cos() * 5.0;
        node.top = Val::Px(100.0 + vertical_offset);
    }
}