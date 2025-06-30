use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct FadeIn {
    timer: Timer,
    end_alpha: f32,
}

#[derive(Component)]
struct Shake {
    timer: Timer,
    intensity: f32,
    original_position: Vec3,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建要淡入的精灵图
    commands.spawn((
        Sprite::from_image(asset_server.load("background/one.png")),
        Transform::from_xyz(0.0, 0.0, 0.0), // 明确设置初始位置
        FadeIn {
            timer: Timer::from_seconds(1.0, TimerMode::Once), // 1秒淡入
            end_alpha: 1.0,
        },
        Shake {
            timer: Timer::from_seconds(9999.0, TimerMode::Once), // 3秒晃动效果
            intensity: 10.0, // 晃动强度（像素）
            original_position: Vec3::new(0.0, 0.0, 0.0),
        }
    ));
}

fn fade_in_system(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut FadeIn)>,
) {
    for (mut sprite, mut fade_in) in query.iter_mut() {
        fade_in.timer.tick(time.delta());

        // 计算当前透明度
        let progress = fade_in.timer.elapsed_secs() / fade_in.timer.duration().as_secs_f32();
        let current_alpha = fade_in.end_alpha * progress;

        // 更新精灵图颜色的透明度
        sprite.color = sprite.color.with_alpha(current_alpha);
    }
}

fn shake_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Shake)>,
) {
    let mut rng = rand::thread_rng();
    
    for (mut transform, mut shake) in query.iter_mut() {
        shake.timer.tick(time.delta());

        if !shake.timer.finished() {
            // 计算晃动衰减（随时间减弱）
            let progress = shake.timer.elapsed_secs() / shake.timer.duration().as_secs_f32();
            let current_intensity = shake.intensity * (1.0 - progress);
            
            // 生成随机偏移
            let offset_x = rng.gen_range(-current_intensity..current_intensity);
            let offset_y = rng.gen_range(-current_intensity..current_intensity);
            
            // 应用晃动效果
            transform.translation = shake.original_position + Vec3::new(offset_x, offset_y, 0.0);
        } else {
            // 晃动结束，恢复原位置
            transform.translation = shake.original_position;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (fade_in_system, shake_system))
        .run();
}