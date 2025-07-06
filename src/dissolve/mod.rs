
use bevy::prelude::*;

// ================== Ren'Py Dissolve 组件 ==================

#[derive(Component)]
pub struct RenpyDissolve {
    pub duration: f32,
    pub elapsed: f32,
    pub from_alpha: f32,
    pub to_alpha: f32,
    pub completed: bool,
}

impl RenpyDissolve {
    pub fn fade_in(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            from_alpha: 0.0,
            to_alpha: 1.0,
            completed: false,
        }
    }
    
    pub fn fade_out(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            from_alpha: 1.0,
            to_alpha: 0.0,
            completed: false,
        }
    }
}

#[derive(Component)]
pub struct RenpyDissolveTransition {
    pub duration: f32,
    pub elapsed: f32,
    pub old_entity: Entity,
    pub new_entity: Entity,
    pub completed: bool,
}

// ================== 核心算法 ==================

// Ren'Py 风格的缓动函数
fn renpy_dissolve_ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    
    if t <= 0.1 {
        // 前10%非常慢，几乎感觉不到变化
        let local_t = t / 0.1;
        let smooth_t = local_t * local_t * (3.0 - 2.0 * local_t); // smoothstep
        smooth_t * 0.05
    } else if t <= 0.8 {
        // 中间70%相对线性但稍有加速
        let local_t = (t - 0.1) / 0.7;
        // 轻微的二次缓动
        let eased_t = local_t * local_t * (3.0 - 2.0 * local_t);
        0.05 + eased_t * 0.85
    } else {
        // 最后20%快速完成
        let local_t = (t - 0.8) / 0.2;
        // 更激进的缓动
        let eased_t = 1.0 - (1.0 - local_t) * (1.0 - local_t);
        0.9 + eased_t * 0.1
    }
}

// 精确的颜色混合
fn precise_alpha_blend(base_alpha: f32, target_alpha: f32, progress: f32) -> f32 {
    let blended = base_alpha + (target_alpha - base_alpha) * progress;
    let quantized = (blended * 255.0).round() / 255.0;
    quantized.clamp(0.0, 1.0)
}

// ================== 系统 ==================

pub fn renpy_dissolve_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut RenpyDissolve, &mut Sprite)>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut dissolve, mut sprite) in query.iter_mut() {
        if dissolve.completed {
            continue;
        }
        
        dissolve.elapsed += delta;
        
        if dissolve.elapsed >= dissolve.duration {
            sprite.color = sprite.color.with_alpha(dissolve.to_alpha);
            dissolve.completed = true;
            commands.entity(entity).remove::<RenpyDissolve>();
        } else {
            let progress = dissolve.elapsed / dissolve.duration;
            let eased_progress = renpy_dissolve_ease(progress);
            
            let current_alpha = precise_alpha_blend(
                dissolve.from_alpha, 
                dissolve.to_alpha, 
                eased_progress
            );
            
            sprite.color = sprite.color.with_alpha(current_alpha);
        }
    }
}

pub fn renpy_dissolve_transition_system(
    time: Res<Time>,
    mut commands: Commands,
    mut transition_query: Query<(Entity, &mut RenpyDissolveTransition)>,
    mut sprite_query: Query<&mut Sprite>,
) {
    let delta = time.delta_secs();
    
    for (entity, mut transition) in transition_query.iter_mut() {
        if transition.completed {
            continue;
        }
        
        transition.elapsed += delta;
        
        if transition.elapsed >= transition.duration {
            if let Ok(mut old_sprite) = sprite_query.get_mut(transition.old_entity) {
                old_sprite.color = old_sprite.color.with_alpha(0.0);
            }
            if let Ok(mut new_sprite) = sprite_query.get_mut(transition.new_entity) {
                new_sprite.color = new_sprite.color.with_alpha(1.0);
            }
            
            transition.completed = true;
            commands.entity(entity).despawn();
        } else {
            let progress = transition.elapsed / transition.duration;
            let eased_progress = renpy_dissolve_ease(progress);
            
            if let Ok(mut old_sprite) = sprite_query.get_mut(transition.old_entity) {
                let alpha = precise_alpha_blend(1.0, 0.0, eased_progress);
                old_sprite.color = old_sprite.color.with_alpha(alpha);
            }
            
            if let Ok(mut new_sprite) = sprite_query.get_mut(transition.new_entity) {
                let alpha = precise_alpha_blend(0.0, 1.0, eased_progress);
                new_sprite.color = new_sprite.color.with_alpha(alpha);
            }
        }
    }
}

// ================== 插件 ==================

pub struct RenpyDissolvePlugin;

impl Plugin for RenpyDissolvePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            renpy_dissolve_system,
            renpy_dissolve_transition_system,
        ));
    }
}