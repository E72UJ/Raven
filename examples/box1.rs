use bevy::prelude::*;

// Fade 组件
#[derive(Component)]
pub struct FadeEffect {
    pub duration: f32,
    pub current_time: f32,
    pub start_alpha: f32,
    pub target_alpha: f32,
    pub is_complete: bool,
}

impl FadeEffect {
    pub fn fade_in(duration: f32) -> Self {
        Self {
            duration,
            current_time: 0.0,
            start_alpha: 0.0,
            target_alpha: 1.0,
            is_complete: false,
        }
    }
    
    pub fn fade_out(duration: f32) -> Self {
        Self {
            duration,
            current_time: 0.0,
            start_alpha: 1.0,
            target_alpha: 0.0,
            is_complete: false,
        }
    }
}

// 设置摄像机
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// 设置角色立绘
fn setup_character(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 生成角色立绘
    commands.spawn((
        Sprite::from_image(asset_server.load("portraits/alice.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(0.5)),
        FadeEffect::fade_in(0.8),
    ));
}

// Fade 系统
fn fade_system(
    time: Res<Time>,
    mut query: Query<(&mut FadeEffect, &mut Sprite)>,
) {
    for (mut fade, mut sprite) in query.iter_mut() {
        if fade.is_complete {
            continue;
        }
        
        fade.current_time += time.delta_secs();
        
        if fade.current_time >= fade.duration {
            fade.current_time = fade.duration;
            fade.is_complete = true;
        }
        
        let progress = fade.current_time / fade.duration;
        let current_alpha = fade.start_alpha + (fade.target_alpha - fade.start_alpha) * progress;
        
        sprite.color.set_alpha(current_alpha);
    }
}

// 触发淡出效果
fn trigger_fadeout(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Sprite>, Without<FadeEffect>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in query.iter() {
            commands.entity(entity).insert(FadeEffect::fade_out(1.0));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_character))
        .add_systems(Update, (fade_system, trigger_fadeout))
        .run();
}