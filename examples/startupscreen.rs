use bevy::prelude::*;

#[derive(Component)]
struct FadeOut {
    timer: Timer,
    start_alpha: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建摄像机
    commands.spawn(Camera2d);
    
    // 添加纯黑色背景
    commands.spawn(Sprite {
        color: Color::BLACK,
        custom_size: Some(Vec2::new(1920.0, 1080.0)), // 设置一个足够大的尺寸覆盖屏幕
        ..default()
    });
    
    // 创建要淡出的文本
    commands.spawn((
        Text2d::new("Hello, 欢迎使用Raven引擎!"),
        TextFont {
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            font_size: 48.0,
            ..default()
        },
        TextColor(Color::WHITE),
        FadeOut {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
            start_alpha: 10.0, // 修正这里，应该是1.0而不是2.0
        }
    ));
}

fn fade_out_system(
    time: Res<Time>,
    mut text_query: Query<(Entity, &mut TextColor, &mut FadeOut), With<Text2d>>,
    mut sprite_query: Query<(Entity, &mut Sprite, &mut FadeOut), Without<Text2d>>,
    mut commands: Commands,
) {
    // 处理文本淡出
    for (entity, mut text_color, mut fade_out) in text_query.iter_mut() {
        fade_out.timer.tick(time.delta());
        
        let progress = fade_out.timer.elapsed_secs() / fade_out.timer.duration().as_secs_f32();
        let current_alpha = fade_out.start_alpha * (1.0 - progress);
        
        text_color.0 = text_color.0.with_alpha(current_alpha);
        
        if fade_out.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
    
    // 处理精灵图淡出
    for (entity, mut sprite, mut fade_out) in sprite_query.iter_mut() {
        fade_out.timer.tick(time.delta());
        
        let progress = fade_out.timer.elapsed_secs() / fade_out.timer.duration().as_secs_f32();
        let current_alpha = fade_out.start_alpha * (1.0 - progress);
        
        sprite.color = sprite.color.with_alpha(current_alpha);
        
        if fade_out.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, fade_out_system)
        .run();
}