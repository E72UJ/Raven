use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ren'Py Style Dissolve Demo".into(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenpyDissolvePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

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

fn renpy_dissolve_system(
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

fn renpy_dissolve_transition_system(
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

// ================== 演示用的标记组件 ==================

#[derive(Component)]
struct DemoSprite(usize); // 用数字标识不同的精灵

#[derive(Component)]
struct InstructionText;

// ================== 设置场景 ==================

fn setup(mut commands: Commands,asset_server: Res<AssetServer>,) {
    // 摄像机
    commands.spawn(Camera2d);
    
    // 创建几个不同颜色的方块作为演示
    let colors = [
        Color::srgb(1.0, 0.2, 0.2), // 红色
        Color::srgb(0.2, 1.0, 0.2), // 绿色
        Color::srgb(0.2, 0.2, 1.0), // 蓝色
        Color::srgb(1.0, 1.0, 0.2), // 黄色
        Color::srgb(1.0, 0.2, 1.0), // 品红
    ];
    
    // 创建演示精灵
    for (i, color) in colors.iter().enumerate() {
        let x_offset = (i as f32 - 2.0) * 150.0;
        
        commands.spawn((
            Sprite {
                image: asset_server.load("characters/protagonist/default.png"),
                // color: Color::srgba(color.to_srgba().red, color.to_srgba().green, color.to_srgba().blue, 0.0), // 初始透明
                custom_size: Some(Vec2::new(699.0, 699.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x_offset, 0.0, 0.0)),
            DemoSprite(i),
        ));
    }
    
    // 添加说明文字（使用 Bevy 0.16 的新 UI 系统）
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        })
        .with_child((
            Text::new(
                "Ren'Py Style Dissolve Demo\n\n\
                Press keys to test different effects:\n\
                1-5: Fade in sprites\n\
                Q-T: Fade out sprites\n\
                SPACE: Fade in all\n\
                ESC: Fade out all\n\
                Z: Dissolve transition between sprites"
            ),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            InstructionText,
        ));
}

// ================== 输入处理 ==================

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    sprite_query: Query<(Entity, &DemoSprite), With<Sprite>>,
    mut transition_counter: Local<usize>,
) {
    // 单个精灵渐入 (数字键 1-5)
    for i in 0..5 {
        let key = match i {
            0 => KeyCode::Digit1,
            1 => KeyCode::Digit2,
            2 => KeyCode::Digit3,
            3 => KeyCode::Digit4,
            4 => KeyCode::Digit5,
            _ => continue,
        };
        
        if keyboard_input.just_pressed(key) {
            for (entity, demo_sprite) in sprite_query.iter() {
                if demo_sprite.0 == i {
                    commands.entity(entity).insert(RenpyDissolve::fade_in(0.5));
                }
            }
        }
    }
    
    // 单个精灵渐出 (Q, W, E, R, T)
    for i in 0..5 {
        let key = match i {
            0 => KeyCode::KeyQ,
            1 => KeyCode::KeyW,
            2 => KeyCode::KeyE,
            3 => KeyCode::KeyR,
            4 => KeyCode::KeyT,
            _ => continue,
        };
        
        if keyboard_input.just_pressed(key) {
            for (entity, demo_sprite) in sprite_query.iter() {
                if demo_sprite.0 == i {
                    commands.entity(entity).insert(RenpyDissolve::fade_out(0.5));
                }
            }
        }
    }
    
    // 全部渐入
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (entity, _) in sprite_query.iter() {
            commands.entity(entity).insert(RenpyDissolve::fade_in(0.5));
        }
    }
    
    // 全部渐出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for (entity, _) in sprite_query.iter() {
            commands.entity(entity).insert(RenpyDissolve::fade_out(0.5));
        }
    }
    
    // dissolve 过渡效果
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        let sprites: Vec<_> = sprite_query.iter().collect();
        if sprites.len() >= 2 {
            let old_idx = *transition_counter % sprites.len();
            let new_idx = (*transition_counter + 1) % sprites.len();
            
            let old_entity = sprites[old_idx].0;
            let new_entity = sprites[new_idx].0;
            
            // 创建过渡实体
            commands.spawn(RenpyDissolveTransition {
                duration: 1.0,
                elapsed: 0.0,
                old_entity,
                new_entity,
                completed: false,
            });
            
            *transition_counter += 1;
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