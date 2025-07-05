// src/transition/fade.rs
use bevy::prelude::*;

#[derive(Component)]
pub struct FadeOverlay {
    pub duration: f32,
    pub elapsed: f32,
    pub fade_type: FadeType,
}

#[derive(Clone, Copy)]
pub enum FadeType {
    FadeIn,   // 从黑到透明
    FadeOut,  // 从透明到黑
}

// 创建渐变覆盖层
pub fn create_fade_overlay(
    commands: &mut Commands,
    fade_type: FadeType,
    duration: f32,
) -> Entity {
    let initial_alpha = match fade_type {
        FadeType::FadeIn => 1.0,  // 开始时不透明
        FadeType::FadeOut => 0.0, // 开始时透明
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, initial_alpha)),
            ZIndex(1000), // 确保在最顶层
            FadeOverlay {
                duration,
                elapsed: 0.0,
                fade_type,
            },
        ))
        .id()
}

// 便捷函数
pub fn fade_in(commands: &mut Commands, duration: f32) -> Entity {
    create_fade_overlay(commands, FadeType::FadeIn, duration)
}

pub fn fade_out(commands: &mut Commands, duration: f32) -> Entity {
    create_fade_overlay(commands, FadeType::FadeOut, duration)
}

// 渐变系统
pub fn fade_system(
    time: Res<Time>,
    mut query: Query<(&mut FadeOverlay, &mut BackgroundColor)>,
) {
    for (mut fade, mut background_color) in query.iter_mut() {
        fade.elapsed += time.delta_secs(); // 修改这里：delta_seconds() -> delta_secs()
        
        let progress = (fade.elapsed / fade.duration).clamp(0.0, 1.0);
        
        let alpha = match fade.fade_type {
            FadeType::FadeIn => 1.0 - progress,  // 从1到0
            FadeType::FadeOut => progress,       // 从0到1
        };
        
        *background_color = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));
    }
}

// 清理完成的渐变
pub fn cleanup_completed_fades(
    mut commands: Commands,
    query: Query<(Entity, &FadeOverlay)>,
) {
    for (entity, fade) in query.iter() {
        if fade.elapsed >= fade.duration {
            commands.entity(entity).despawn();
        }
    }
}