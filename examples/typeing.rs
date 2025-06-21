use bevy::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_typewriter, update_ui))
        .run();
}

#[derive(Resource)]
struct UiElement {
    entity: Entity,
}

// 打字机组件
#[derive(Component)]
struct Typewriter {
    full_text: String,           // 完整文本
    current_index: usize,        // 当前显示到第几个字符
    timer: Timer,                // 控制打字速度的计时器
    is_finished: bool,           // 是否完成打字
}

impl Typewriter {
    fn new(text: String, chars_per_second: f32) -> Self {
        let delay = Duration::from_secs_f32(1.0 / chars_per_second);
        Self {
            full_text: text,
            current_index: 0,
            timer: Timer::new(delay, TimerMode::Repeating),
            is_finished: false,
        }
    }
    
    fn get_current_text(&self) -> String {
        if self.current_index >= self.full_text.len() {
            return self.full_text.clone();
        }
        
        // 正确处理UTF-8字符
        self.full_text.chars().take(self.current_index).collect()
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 创建相机
    commands.spawn(Camera2d);

    // 要显示的完整文本
    let full_text = "这是一个带图片背景的文本框！\n可以显示多行文本。\n现在有了打字机效果！".to_string();

    // 创建带图片背景的文本框容器
    let textbox_container = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(190.0),
            position_type: PositionType::Absolute,
            top: Val::Px(470.0),
            ..default()
        })
        .insert(ImageNode::new(asset_server.load("textures/background.png")))
        .insert(BorderColor(Color::WHITE))
        .with_children(|parent| {
            // 文本内容 - 添加打字机组件
            parent.spawn((
                Text::new(""), // 初始为空文本
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    position_type: PositionType::Relative,
                    left: Val::Px(240.0),
                    top: Val::Px(30.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                // 添加打字机组件，每秒显示20个字符
                Typewriter::new(full_text, 20.0),
            ));
        })
        .id();

    // 存储UI元素实体
    commands.insert_resource(UiElement { entity: textbox_container });
}

// 打字机系统
fn update_typewriter(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut Typewriter)>,
) {
    for (mut text, mut typewriter) in query.iter_mut() {
        // 如果已经完成，跳过
        if typewriter.is_finished {
            continue;
        }
        
        // 更新计时器
        typewriter.timer.tick(time.delta());
        
        // 当计时器完成时，显示下一个字符
        if typewriter.timer.just_finished() {
            typewriter.current_index += 1;
            
            // 更新显示的文本
            text.0 = typewriter.get_current_text();
            
            // 检查是否完成
            if typewriter.current_index >= typewriter.full_text.chars().count() {
                typewriter.is_finished = true;
                // 确保显示完整文本
                text.0 = typewriter.full_text.clone();
            }
        }
    }
}

fn update_ui(
    time: Res<Time>,
    ui_element: Res<UiElement>,
    mut query: Query<&mut Node>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut typewriter_query: Query<&mut Typewriter>,
) {
    // 按空格键重置打字机效果
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut typewriter in typewriter_query.iter_mut() {
            typewriter.current_index = 0;
            typewriter.is_finished = false;
            typewriter.timer.reset();
        }
    }
}