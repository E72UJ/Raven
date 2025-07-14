use bevy::prelude::*;

#[derive(Component)]
struct TweenAnimation {
    start_pos: Vec3,
    end_pos: Vec3,
    start_scale: Vec3,
    end_scale: Vec3,
    start_color: Color,
    end_color: Color,
    duration: f32,
    elapsed: f32,
    is_reverse: bool,
}

impl TweenAnimation {
    fn new(start_pos: Vec3, end_pos: Vec3, duration: f32) -> Self {
        Self {
            start_pos,
            end_pos,
            start_scale: Vec3::ONE,
            end_scale: Vec3::ONE * 1.5,
            start_color: Color::WHITE,
            end_color: Color::srgb(1.0, 0.0, 0.0), // 使用 srgb 代替 RED
            duration,
            elapsed: 0.0,
            is_reverse: false,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            update_tween_animation,
            handle_input,
        ))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 生成相机
    commands.spawn(Camera2d);
    
    // 生成一个精灵，带有tween动画组件
    commands.spawn((
        Sprite {
            image: asset_server.load("characters/heroine/default.png"), // 确保你有这个图片文件
            color: Color::WHITE,
            ..default()
        },
        Transform::from_xyz(-200.0, 0.0, 0.0),
        TweenAnimation::new(
            Vec3::new(-200.0, 0.0, 0.0),
            Vec3::new(200.0, 100.0, 0.0),
            2.0,
        ),
    ));
    
    // 生成第二个精灵做对比
    commands.spawn((
        Sprite {
            image: asset_server.load("icon.png"),
            color: Color::srgb(0.0, 0.0, 1.0), // 使用 srgb 代替 BLUE
            ..default()
        },
        Transform::from_xyz(0.0, -150.0, 0.0),
        TweenAnimation::new(
            Vec3::new(0.0, -110.0, 0.0),
            Vec3::new(0.0, 150.0, 0.0),
            1.5,
        ),
    ));
}

fn update_tween_animation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &mut TweenAnimation)>,
) {
    for (mut transform, mut sprite, mut tween) in query.iter_mut() {
        tween.elapsed += time.delta_secs(); // 使用 delta_secs 代替 delta_seconds
        
        // 计算进度 (0.0 到 1.0)
        let mut progress = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        
        // 如果是反向播放，翻转进度
        if tween.is_reverse {
            progress = 1.0 - progress;
        }
        
        // 使用缓动函数（这里使用 ease-in-out）
        let eased_progress = ease_in_out_cubic(progress);
        
        // 插值位置
        transform.translation = tween.start_pos.lerp(tween.end_pos, eased_progress);
        
        // 插值缩放
        transform.scale = tween.start_scale.lerp(tween.end_scale, eased_progress);
        
        // 插值颜色
        sprite.color = tween.start_color.mix(&tween.end_color, eased_progress);
        
        // 如果动画完成，反转方向
        if tween.elapsed >= tween.duration {
            tween.is_reverse = !tween.is_reverse;
            tween.elapsed = 0.0;
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TweenAnimation>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        // 按空格重置所有动画
        for mut tween in query.iter_mut() {
            tween.elapsed = 0.0;
            tween.is_reverse = false;
        }
    }
    
    if keyboard.just_pressed(KeyCode::KeyR) {
        // 按R键反转动画方向
        for mut tween in query.iter_mut() {
            tween.is_reverse = !tween.is_reverse;
        }
    }
}

// 缓动函数：ease-in-out cubic
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

// 其他常用的缓动函数
#[allow(dead_code)]
fn ease_in_quad(t: f32) -> f32 {
    t * t
}

#[allow(dead_code)]
fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

#[allow(dead_code)]
fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

#[allow(dead_code)]
fn bounce_ease_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;
    
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}