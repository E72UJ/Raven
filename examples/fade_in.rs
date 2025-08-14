use bevy::prelude::*;

#[derive(Component)]
struct FadeIn {
    timer: Timer,
    end_alpha: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建摄像机
    commands.spawn(Camera2d);

    // 创建要淡入的精灵图
    commands.spawn((
        Sprite::from_image(asset_server.load("background/one.png")),
        
        FadeIn {
            timer: Timer::from_seconds(3.0, TimerMode::Once), // 2秒淡入
            end_alpha: 1.0,
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

        // 淡入完成
        if fade_in.timer.finished() {
            // 可以在这里执行一些后续操作
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, fade_in_system)
        .run();
}