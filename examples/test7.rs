use bevy::prelude::*;
use std::time::Duration;

// 角色组件
#[derive(Component)]
pub struct Character {
    pub side: CharacterSide,
}

#[derive(PartialEq, Debug, Clone)]
pub enum CharacterSide {
    Left,
    Right,
}

// 移动组件
#[derive(Component)]
pub struct Movement {
    pub velocity: Vec2,
    pub speed: f32,
}

// 碰撞器组件
#[derive(Component)]
pub struct Collider {
    pub radius: f32,
    pub mass: f32,
}

// 震动效果组件
#[derive(Component)]
pub struct ShakeEffect {
    pub intensity: f32,
    pub duration: Timer,
    pub original_position: Vec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            input_system,
            auto_movement_system,
            movement_system,
            collision_system,
            shake_system,
        ).chain())
        .run();
}

fn setup(mut commands: Commands) {
    // 设置摄像机
    commands.spawn(Camera2d);

    // 左侧角色 (蓝色圆形)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(80.0, 80.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-300.0, 0.0, 0.0)),
        Character {
            side: CharacterSide::Left,
        },
        Movement {
            velocity: Vec2::ZERO,
            speed: 200.0,
        },
        Collider {
            radius: 40.0,
            mass: 1.0,
        },
    ));

    // 右侧角色 (红色圆形)
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(80.0, 80.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(300.0, 0.0, 0.0)),
        Character {
            side: CharacterSide::Right,
        },
        Movement {
            velocity: Vec2::new(-100.0, 0.0),
            speed: 150.0,
        },
        Collider {
            radius: 40.0,
            mass: 1.2,
        },
    ));
}

// 输入系统
fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Movement, &Character)>,
) {
    for (mut movement, character) in query.iter_mut() {
        if character.side == CharacterSide::Left {
            movement.velocity = Vec2::ZERO;
            
            if keyboard_input.pressed(KeyCode::KeyA) {
                movement.velocity.x = -1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                movement.velocity.x = 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyW) {
                movement.velocity.y = 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                movement.velocity.y = -1.0;
            }
        }
    }
}

// 自动移动系统
fn auto_movement_system(
    mut query: Query<(&mut Movement, &Transform, &Character)>,
) {
    for (mut movement, transform, character) in query.iter_mut() {
        if character.side == CharacterSide::Right {
            // 简单AI：碰到边界就反弹
            if transform.translation.x <= -200.0 && movement.velocity.x < 0.0 {
                movement.velocity.x = movement.velocity.x.abs();
            } else if transform.translation.x >= 350.0 && movement.velocity.x > 0.0 {
                movement.velocity.x = -movement.velocity.x.abs();
            }
        }
    }
}

// 移动系统 - 修复版：移除震动过滤器，让角色能正常移动
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Movement), Without<ShakeEffect>>,
) {
    for (mut transform, movement) in query.iter_mut() {
        let velocity = movement.velocity.normalize_or_zero() * movement.speed;
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        
        // 边界限制
        transform.translation.x = transform.translation.x.clamp(-400.0, 400.0);
        transform.translation.y = transform.translation.y.clamp(-300.0, 300.0);
    }
}

// 碰撞系统
fn collision_system(
    mut commands: Commands,
    mut characters: Query<(Entity, &mut Transform, &mut Movement, &Collider, &Character)>,
) {
    let mut collisions = Vec::new();
    
    // 收集所有碰撞信息
    let char_data: Vec<_> = characters.iter()
        .map(|(entity, transform, movement, collider, character)| {
            (entity, transform.translation, movement.velocity, collider.radius, collider.mass, character.side.clone())
        })
        .collect();
    
    // 检测碰撞
    for i in 0..char_data.len() {
        for j in (i + 1)..char_data.len() {
            let (e1, pos1, vel1, r1, m1, _) = &char_data[i];
            let (e2, pos2, vel2, r2, m2, _) = &char_data[j];
            
            let distance = pos1.distance(*pos2);
            let min_distance = r1 + r2;
            
            if distance < min_distance && distance > 0.1 {
                let normal = (*pos2 - *pos1).normalize_or_zero();
                let overlap = min_distance - distance;
                
                collisions.push((*e1, *e2, normal, overlap, *vel1, *vel2, *m1, *m2));
            }
        }
    }
    
    // 处理碰撞
    for (e1, e2, normal, overlap, vel1, vel2, m1, m2) in collisions {
        // 分离物体
        let separation = normal * overlap * 0.5;
        
        if let Ok((_, mut t1, mut mv1, _, _)) = characters.get_mut(e1) {
            t1.translation.x -= separation.x;
            t1.translation.y -= separation.y;
            
            // 计算反弹
            let relative_vel = vel1 - vel2;
            let vel_along_normal = relative_vel.dot(normal.truncate());
            
            if vel_along_normal < 0.0 {
                let restitution = 0.7;
                let impulse_magnitude = -(1.0 + restitution) * vel_along_normal / (1.0/m1 + 1.0/m2);
                let impulse = normal.truncate() * impulse_magnitude;
                
                mv1.velocity += impulse / m1;
                
                // 添加震动
                commands.entity(e1).insert(ShakeEffect {
                    intensity: 10.0,
                    duration: Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once),
                    original_position: t1.translation,
                });
            }
        }
        
        if let Ok((_, mut t2, mut mv2, _, _)) = characters.get_mut(e2) {
            t2.translation.x += separation.x;
            t2.translation.y += separation.y;
            
            // 计算反弹
            let relative_vel = vel2 - vel1;
            let vel_along_normal = relative_vel.dot((-normal).truncate());
            
            if vel_along_normal < 0.0 {
                let restitution = 0.7;
                let impulse_magnitude = -(1.0 + restitution) * vel_along_normal / (1.0/m1 + 1.0/m2);
                let impulse = (-normal).truncate() * impulse_magnitude;
                
                mv2.velocity += impulse / m2;
                
                // 添加震动
                commands.entity(e2).insert(ShakeEffect {
                    intensity: 10.0,
                    duration: Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once),
                    original_position: t2.translation,
                });
            }
        }
    }
}

// 震动系统 - 修复版：让有震动效果的角色也能继续移动
fn shake_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut ShakeEffect)>,
    movement_query: Query<&Movement>,
) {
    for (entity, mut transform, mut shake) in query.iter_mut() {
        shake.duration.tick(time.delta());
        
        // 先更新原始位置（跟随正常移动）
        if let Ok(movement) = movement_query.get(entity) {
            let velocity = movement.velocity.normalize_or_zero() * movement.speed;
            shake.original_position.x += velocity.x * time.delta_secs();
            shake.original_position.y += velocity.y * time.delta_secs();
            
            // 边界限制
            shake.original_position.x = shake.original_position.x.clamp(-400.0, 400.0);
            shake.original_position.y = shake.original_position.y.clamp(-300.0, 300.0);
        }
        
        if shake.duration.finished() {
            transform.translation = shake.original_position;
            commands.entity(entity).remove::<ShakeEffect>();
        } else {
            let progress = shake.duration.elapsed_secs() / shake.duration.duration().as_secs_f32();
            let intensity = shake.intensity * (1.0 - progress);
            
            let shake_x = (time.elapsed_secs() * 25.0).sin() * intensity;
            let shake_y = (time.elapsed_secs() * 30.0).cos() * intensity;
            
            transform.translation = shake.original_position + Vec3::new(shake_x, shake_y, 0.0);
        }
    }
}