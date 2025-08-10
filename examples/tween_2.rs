use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
struct SwayingImage {
    amplitude: f32,    // 摇晃幅度
    frequency: f32,    // 摇晃频率
    timer: f32,        // 内部计时器
    original_x: f32,   // 原始X位置
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sway_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    // 加载并生成摇晃的图片
    commands.spawn((
        Sprite::from_image(asset_server.load("images/sylvie green normal.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        SwayingImage {
            amplitude: 10.0,   // 左右摇晃50像素
            frequency: 2.0,    // 每秒2次循环
            timer: 0.0,
            original_x: 0.0,   // 原始位置
        },
    ));
}

fn sway_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut SwayingImage)>,
) {
    for (mut transform, mut swaying) in query.iter_mut() {
        swaying.timer += time.delta_secs();
        
        // 使用正弦函数计算摇晃位置
        let offset = (swaying.timer * swaying.frequency * 2.0 * PI).sin() * swaying.amplitude;
        transform.translation.x = swaying.original_x + offset;
    }
}
