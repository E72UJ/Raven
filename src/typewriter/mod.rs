use bevy::prelude::*;

// 打字机文本组件
#[derive(Component)]
pub struct TypewriterText {
    pub full_text: String,
    pub current_length: usize,
    pub timer: Timer,
    pub is_active: bool, // 添加这个字段
}

impl TypewriterText {
    /// 创建新的打字机文本组件
    pub fn new(text: String, chars_per_second: f32) -> Self {
        Self {
            full_text: text,
            current_length: 0,
            timer: Timer::from_seconds(1.0 / chars_per_second, TimerMode::Repeating),
            is_active: false, // 初始化为 false
        }
    }

    /// 重置打字机效果
    pub fn reset(&mut self, new_text: String) {
        self.full_text = new_text;
        self.current_length = 0;
        self.timer.reset();
    }

    /// 检查是否完成显示
    pub fn is_finished(&self) -> bool {
        self.current_length >= self.full_text.len()
    }

    /// 立即显示全部文本
    pub fn show_all(&mut self) {
        self.current_length = self.full_text.len();
    }
}

// 打字机系统
pub fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TypewriterText)>,
) {
    for (mut text, mut typewriter) in query.iter_mut() {
        if typewriter.is_active && !typewriter.full_text.is_empty() {
            typewriter.timer.tick(time.delta());
            
            if typewriter.timer.just_finished() && typewriter.current_length < typewriter.full_text.len() {
                typewriter.current_length += 1;
                text.0 = typewriter.full_text.chars().take(typewriter.current_length).collect();
                
                println!("打字机更新显示: '{}'", text.0);
                
                // 如果完成了，标记为不活跃
                if typewriter.current_length >= typewriter.full_text.len() {
                    typewriter.is_active = false;
                }
            }
        }
    }
}

// 打字机插件
pub struct TypewriterPlugin;

impl Plugin for TypewriterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, typewriter_system);
    }
}

// 辅助函数：为文本实体添加打字机效果
pub fn add_typewriter_effect(
    commands: &mut Commands,
    entity: Entity,
    text: String,
    chars_per_second: f32,
) {
    commands.entity(entity).insert(TypewriterText::new(text, chars_per_second));
}