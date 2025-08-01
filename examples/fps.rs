use bevy::prelude::*;

#[derive(Component)]
struct AnimationTimer {
    timer: Timer,
    frames: Vec<Handle<Image>>,
    current_frame: usize,
}

impl AnimationTimer {
    fn new(frames: Vec<Handle<Image>>, fps: f32) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            frames,
            current_frame: 0,
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载所有帧图片
    let mut frames = Vec::new();
    for i in 1..=60 {
        let path = format!("fps/frame_{}.png", i);
        frames.push(asset_server.load(path));
    }
    
    // 创建2D摄像机 - 尝试这种方式
    commands.spawn(Camera2d);
    
    // 创建动画精灵，从第一帧开始
    commands.spawn((
        Sprite::from_image(frames[0].clone()),
       Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::new(0.5, 0.5, 1.0)), // 放大2倍
        AnimationTimer::new(frames, 30.0),
    ));
}

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        animation.timer.tick(time.delta());
        
        if animation.timer.just_finished() {
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "帧动画播放器".into(),
                resolution: (800.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprites)
        .run();
}
