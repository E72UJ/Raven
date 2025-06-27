use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct DissolveParticle {
    timer: Timer,
    original_position: Vec3,
    velocity: Vec3,
    life_progress: f32,
    dissolve_delay: f32,
}

#[derive(Component)]
struct MontageImage {
    timer: Timer,
    image_index: usize,
    total_images: usize,
}

const PARTICLE_COUNT: usize = 2000; // 溶解粒子数量
const MONTAGE_IMAGES: &[&str] = &[
    "background/one.png",
    "background/two.png", 
    "background/three.png",
    "background/four.png",
]; // 蒙太奇图像列表

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建主要的蒙太奇图像
    commands.spawn((
        Sprite::from_image(asset_server.load(MONTAGE_IMAGES[0])),
        Transform::from_xyz(0.0, 0.0, 1.0),
        MontageImage {
            timer: Timer::from_seconds(3.0, TimerMode::Repeating), // 每3秒切换图像
            image_index: 0,
            total_images: MONTAGE_IMAGES.len(),
        },
    ));

    let mut rng = rand::thread_rng();
    
    // 创建溶解粒子
    for _ in 0..PARTICLE_COUNT {
        // 随机分布在图像区域内
        let pos_x = rng.gen_range(-400.0..400.0);
        let pos_y = rng.gen_range(-300.0..300.0);
        let position = Vec3::new(pos_x, pos_y, 0.0);

        // 随机的溶解方向和速度
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(50.0..150.0);
        let velocity = Vec3::new(
            angle.cos() * speed,
            angle.sin() * speed,
            0.0,
        );

        commands.spawn((
            Sprite {
                color: Color::srgba(
                    rng.gen_range(0.8..1.0),
                    rng.gen_range(0.8..1.0), 
                    rng.gen_range(0.8..1.0),
                    0.0, // 初始透明
                ),
                ..default()
            },
            Transform {
                translation: position,
                scale: Vec3::splat(rng.gen_range(2.0..8.0)),
                ..default()
            },
            DissolveParticle {
                timer: Timer::from_seconds(4.0, TimerMode::Once),
                original_position: position,
                velocity,
                life_progress: 0.0,
                dissolve_delay: rng.gen_range(0.0..2.0),
            },
        ));
    }
}

fn montage_system(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Sprite, &mut MontageImage)>,
) {
    for (mut sprite, mut montage) in query.iter_mut() {
        montage.timer.tick(time.delta());
        
        if montage.timer.just_finished() {
            // 切换到下一张图像
            montage.image_index = (montage.image_index + 1) % montage.total_images;
            sprite.image = asset_server.load(MONTAGE_IMAGES[montage.image_index]);
            
            // 添加切换时的透明度动画
            let switch_progress = montage.timer.elapsed_secs() / montage.timer.duration().as_secs_f32();
            sprite.color = Color::WHITE.with_alpha(1.0 - switch_progress);
        }
        
        // 图像切换时的淡入淡出效果
        let switch_progress = montage.timer.elapsed_secs() / montage.timer.duration().as_secs_f32();
        if switch_progress < 0.2 {
            // 淡入新图像
            let fade_alpha = switch_progress / 0.2;
            sprite.color = Color::WHITE.with_alpha(fade_alpha);
        } else if switch_progress > 0.8 {
            // 淡出旧图像
            let fade_alpha = (1.0 - switch_progress) / 0.2;
            sprite.color = Color::WHITE.with_alpha(fade_alpha);
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn dissolve_system(
    time: Res<Time>,
    mut particle_query: Query<(&mut Transform, &mut Sprite, &mut DissolveParticle)>,
    montage_query: Query<&MontageImage>,
) {
    // 获取当前蒙太奇的切换进度来同步粒子效果
    let montage_progress = if let Ok(montage) = montage_query.get_single() {
        montage.timer.elapsed_secs() / montage.timer.duration().as_secs_f32()
    } else {
        0.0
    };

    for (mut transform, mut sprite, mut particle) in particle_query.iter_mut() {
        particle.timer.tick(time.delta());
        
        let elapsed = particle.timer.elapsed_secs();
        
        // 延迟开始溶解效果
        if elapsed > particle.dissolve_delay {
            let life_time = elapsed - particle.dissolve_delay;
            let duration = particle.timer.duration().as_secs_f32() - particle.dissolve_delay;
            particle.life_progress = (life_time / duration).min(1.0);
            
            // 位置更新：粒子向外扩散
            let expansion_factor = 1.0 + particle.life_progress * 2.0;
            transform.translation = particle.original_position 
                + particle.velocity * life_time * expansion_factor;
            
            // 透明度变化：先出现再消失
            let alpha = if particle.life_progress < 0.3 {
                // 快速出现
                particle.life_progress / 0.3
            } else if particle.life_progress < 0.7 {
                // 保持可见
                1.0
            } else {
                // 逐渐消失
                1.0 - (particle.life_progress - 0.7) / 0.3
            };
            
            // 根据蒙太奇进度调整颜色
            let color_shift = montage_progress;
            let r = 0.8 + 0.2 * (color_shift * std::f32::consts::TAU).sin();
            let g = 0.8 + 0.2 * ((color_shift + 0.33) * std::f32::consts::TAU).sin();
            let b = 0.8 + 0.2 * ((color_shift + 0.66) * std::f32::consts::TAU).sin();
            
            sprite.color = Color::srgba(r, g, b, alpha);
            
            // 尺寸变化：先增大再缩小
            let scale = if particle.life_progress < 0.5 {
                2.0 + particle.life_progress * 4.0
            } else {
                6.0 - (particle.life_progress - 0.5) * 8.0
            };
            transform.scale = Vec3::splat(scale.max(0.1));
        }
        
        // 重置已完成的粒子
        if particle.timer.finished() {
            particle.timer.reset();
            particle.life_progress = 0.0;
            sprite.color = sprite.color.with_alpha(0.0);
            transform.translation = particle.original_position;
        }
    }
}

// 手动触发新一轮溶解效果
fn trigger_dissolve_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut DissolveParticle>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for mut particle in query.iter_mut() {
            particle.timer.reset();
            particle.life_progress = 0.0;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            montage_system,
            dissolve_system,
            trigger_dissolve_system,
        ))
        .run();
}