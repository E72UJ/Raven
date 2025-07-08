use bevy::prelude::*;

// 打字机组件
#[derive(Component)]
pub struct TypewriterText {
    pub full_text: String,
    pub current_length: usize,
    pub timer: Timer,
    pub is_finished: bool,
}

impl TypewriterText {
    // 创建新的打字机文本
    pub fn new(text: String, speed_ms: u64) -> Self {
        Self {
            full_text: text,
            current_length: 0,
            timer: Timer::from_seconds(speed_ms as f32 / 1000.0, TimerMode::Repeating),
            is_finished: false,
        }
    }
    
    // 设置新文本
    pub fn set_text(&mut self, text: String) {
        self.full_text = text;
        self.current_length = 0;
        self.is_finished = false;
        self.timer.reset();
    }
    
    // 立即显示全部文本
    pub fn skip_to_end(&mut self) {
        self.current_length = self.full_text.len();
        self.is_finished = true;
    }
    
    // 检查是否完成显示
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
    
    // 获取当前显示的文本
    pub fn get_current_text(&self) -> String {
        self.full_text.chars().take(self.current_length).collect()
    }
}

// 打字机事件
#[derive(Event)]
pub enum TypewriterEvent {
    TextFinished(Entity),  // 文本显示完成
    CharacterShown(Entity, char),  // 显示了新字符
}

// 打字机系统
pub fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Text, &mut TypewriterText)>,
    mut event_writer: EventWriter<TypewriterEvent>,
) {
    for (entity, mut text, mut typewriter) in query.iter_mut() {
        if typewriter.is_finished {
            continue;
        }
        
        typewriter.timer.tick(time.delta());
        
        if typewriter.timer.just_finished() && typewriter.current_length < typewriter.full_text.len() {
            typewriter.current_length += 1;
            
            // 获取当前字符
            if let Some(current_char) = typewriter.full_text.chars().nth(typewriter.current_length - 1) {
                event_writer.send(TypewriterEvent::CharacterShown(entity, current_char));
            }
            
            // 更新显示的文本
            text.0 = typewriter.get_current_text();
            
            // 检查是否完成
            if typewriter.current_length >= typewriter.full_text.len() {
                typewriter.is_finished = true;
                event_writer.send(TypewriterEvent::TextFinished(entity));
            }
        }
    }
}

// 处理跳过打字机效果的系统
pub fn typewriter_skip_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut TypewriterText>,
) {
    // 按空格键或点击鼠标跳过
    if keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        for mut typewriter in query.iter_mut() {
            if !typewriter.is_finished {
                typewriter.skip_to_end();
            }
        }
    }
}

// 打字机插件
pub struct TypewriterPlugin;

impl Plugin for TypewriterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<TypewriterEvent>()
            .add_systems(Update, (
                typewriter_system,
                typewriter_skip_system,
            ));
    }
}