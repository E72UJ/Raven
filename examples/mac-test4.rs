use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Fragment {
    timer: Timer,
    original_position: Vec3,
    velocity: Vec3,
    rotation_speed: f32,
    fade_delay: f32,
}

#[derive(Component)]
struct FragmentContainer;

const GRID_SIZE: usize = 8; // 8x8的网格，共64个片段

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建容器来管理所有片段
    let container = commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        FragmentContainer,
    )).id();

    let mut rng = rand::thread_rng();
    
    // 创建破碎片段
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            // 计算片段位置
            let fragment_size = 64.0; // 每个片段的大小
            let total_size = GRID_SIZE as f32 * fragment_size;
            let start_x = -total_size / 2.0 + fragment_size / 2.0;
            let start_y = -total_size / 2.0 + fragment_size / 2.0;
            
            let pos_x = start_x + x as f32 * fragment_size;
            let pos_y = start_y + y as f32 * fragment_size;
            let position = Vec3::new(pos_x, pos_y, 0.0);

            // 生成随机的破碎效果参数
            let velocity = Vec3::new(
                rng.gen_range(-200.0..200.0),
                rng.gen_range(-200.0..200.0),
                0.0,
            );
            
            let rotation_speed = rng.gen_range(-5.0..5.0);
            let fade_delay = rng.gen_range(0.0..2.0);

            let fragment = commands.spawn((
                Sprite {
                    image: asset_server.load("background/one.png"),
                    // 使用纹理图集来显示图像的一部分
                    texture_atlas: Some(TextureAtlas {
                        layout: asset_server.add(TextureAtlasLayout::from_grid(
                            UVec2::splat(64), // 假设每个片段是64x64像素
                            GRID_SIZE as u32,
                            GRID_SIZE as u32,
                            None,
                            None,
                        )),
                        index: y * GRID_SIZE + x,
                    }),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_translation(position),
                Fragment {
                    timer: Timer::from_seconds(5.0, TimerMode::Once),
                    original_position: position,
                    velocity,
                    rotation_speed,
                    fade_delay,
                },
            )).id();

            commands.entity(container).add_child(fragment);
        }
    }
}

fn fragment_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut Fragment)>,
) {
    for (mut transform, mut sprite, mut fragment) in query.iter_mut() {
        fragment.timer.tick(time.delta());
        
        let elapsed = fragment.timer.elapsed_secs();
        let duration = fragment.timer.duration().as_secs_f32();
        
        // 延迟开始破碎效果
        if elapsed > fragment.fade_delay {
            let progress = (elapsed - fragment.fade_delay) / (duration - fragment.fade_delay);
            
            // 位置变化：片段飞散
            let gravity = Vec3::new(0.0, -300.0, 0.0); // 重力效果
            let time_since_start = elapsed - fragment.fade_delay;
            
            transform.translation = fragment.original_position 
                + fragment.velocity * time_since_start
                + 0.5 * gravity * time_since_start * time_since_start;
            
            // 旋转效果
            transform.rotation *= Quat::from_rotation_z(fragment.rotation_speed * time.delta_secs());
            
            // 缩放效果：逐渐缩小
            let scale = 1.0 - progress * 0.3;
            transform.scale = Vec3::splat(scale.max(0.1));
            
            // 透明度变化：逐渐消失
            let alpha = (1.0 - progress).max(0.0);
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

// 可选：添加触发破碎效果的系统
fn trigger_shatter_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Fragment>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // 重置所有片段的计时器
        for mut fragment in query.iter_mut() {
            fragment.timer.reset();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (fragment_system, trigger_shatter_system))
        .run();
}