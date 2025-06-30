use bevy::prelude::*;

#[derive(Component)]
struct TypewriterText {
    full_text: String,
    current_index: usize,
    timer: Timer,
    is_playing: bool,
    is_finished: bool,
}

impl TypewriterText {
    fn new(text: String, chars_per_second: f32) -> Self {
        Self {
            full_text: text,
            current_index: 0,
            timer: Timer::from_seconds(1.0 / chars_per_second, TimerMode::Repeating),
            is_playing: true,
            is_finished: false,
        }
    }
    
    fn skip_to_end(&mut self) {
        self.current_index = self.full_text.chars().count();
        self.is_finished = true;
    }
}

fn setup(mut commands: Commands,  asset_server: Res<AssetServer>,) {
    commands.spawn(Camera2d);
    
    // 创建打字机文本
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 40.0,
            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
            ..default()
        },
        TextColor(Color::WHITE),
        TypewriterText::new("你好 这是一个打字机哈哈哈啊啊哈啊啊啊哈啊哈哈啊啊啊 啊啊啊啊 啊啊啊 啊啊".to_string(), 1.0),
    ));
}

fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TypewriterText)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (mut text, mut typewriter) in query.iter_mut() {
        // 空格键跳过
        if keyboard.just_pressed(KeyCode::Space) {
            typewriter.skip_to_end();
        }
        
        if !typewriter.is_playing || typewriter.is_finished {
            if typewriter.is_finished {
                let chars: Vec<char> = typewriter.full_text.chars().collect();
                **text = chars.iter().collect();
            }
            continue;
        }
        
        typewriter.timer.tick(time.delta());
        
        if typewriter.timer.just_finished() {
            let chars: Vec<char> = typewriter.full_text.chars().collect();
            
            if typewriter.current_index < chars.len() {
                typewriter.current_index += 1;
                let displayed_text: String = chars[..typewriter.current_index].iter().collect();
                **text = displayed_text;
            } else {
                typewriter.is_finished = true;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, typewriter_system)
        .run();
}