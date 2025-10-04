use bevy::prelude::*;
use std::time::Duration;

// 将文本分页，每页显示几行
const TYPEWRITER_PAGES: &[&[&str]] = &[
    &[
        "我已经错过那最好的年华",
        "我已经错过那最好的年代", 
        "我也早已错过那最好的事",
    ],
    &[
        "----最好的事是从未存在",
        "----其次是现在就死去",
    ],
    &[
        "歌唱自由吧，尽管它虚伪至极",
        "我疑惑你们为何将它放在第一的位置",
        "我不明白，或许也没有学到",
    ],
    &[
        "但当伊曼努尔的例子从你的口中脱出时，我突然笑了",
        "原来这就是自由",
        "原来我，正要践行的，是自由啊",
    ],
];

// 打字机状态组件
#[derive(Component)]
pub struct TypewriterText {
    pub current_page: usize,
    pub current_line: usize,
    pub current_char: usize,
    pub timer: Timer,
    pub char_delay: Duration,
    pub line_delay: Duration,
    pub page_delay: Duration,
    pub is_complete: bool,
    pub waiting_for_next_page: bool,
}

impl Default for TypewriterText {
    fn default() -> Self {
        Self {
            current_page: 0,
            current_line: 0,
            current_char: 0,
            timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
            char_delay: Duration::from_millis(50),
            line_delay: Duration::from_millis(800),
            page_delay: Duration::from_millis(2000), // 页面间停顿
            is_complete: false,
            waiting_for_next_page: false,
        }
    }
}

// 渐暗效果组件
#[derive(Component)]
pub struct FadeEffect {
    pub timer: Timer,
    pub is_fading_out: bool,
    pub is_fading_in: bool,
}

impl Default for FadeEffect {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(800), TimerMode::Once),
            is_fading_out: false,
            is_fading_in: false,
        }
    }
}

// 插件结构
pub struct TypewriterPlugin;

impl Plugin for TypewriterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_typewriter)
            .add_systems(Update, (typewriter_system, fade_system));
    }
}

// 设置打字机文本
fn setup_typewriter(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 摄像机
    commands.spawn(Camera2d);

    // 加载字体
    let font = asset_server.load("fonts/ark.ttf");

    // 创建全屏容器来实现居中
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            // 在容器中创建文本
            parent.spawn((
                Text::new(""),
                TextFont {
                    font: font,
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TypewriterText::default(),
                FadeEffect::default(),
            ));
        });
}



// 打字机效果系统
fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TypewriterText, &mut FadeEffect)>,
) {
    for (mut text, mut typewriter, mut fade) in query.iter_mut() {
        if typewriter.is_complete {
            continue;
        }

        // 如果正在等待下一页且不在渐变中
        if typewriter.waiting_for_next_page && !fade.is_fading_out && !fade.is_fading_in {
            typewriter.timer.tick(time.delta());
            
            if typewriter.timer.just_finished() {
                // 开始渐出效果
                fade.is_fading_out = true;
                fade.timer.reset();
                typewriter.waiting_for_next_page = false;
            }
            continue;
        }

        // 如果不在渐变过程中，继续打字
        if !fade.is_fading_out && !fade.is_fading_in {
            typewriter.timer.tick(time.delta());

            if typewriter.timer.just_finished() {
                if typewriter.current_page >= TYPEWRITER_PAGES.len() {
                    typewriter.is_complete = true;
                    continue;
                }

                let current_page = TYPEWRITER_PAGES[typewriter.current_page];
                
                if typewriter.current_line >= current_page.len() {
                    // 当前页完成，准备切换到下一页
                    typewriter.waiting_for_next_page = true;
                    let page_delay = typewriter.page_delay;
                    typewriter.timer.set_duration(page_delay);
                    continue;
                }

                let current_line_text = current_page[typewriter.current_line];
                let chars: Vec<char> = current_line_text.chars().collect();
                
                if typewriter.current_char < chars.len() {
                    // 显示到当前字符
                    let mut full_text = String::new();
                    
                    // 添加当前页已完成的行
                    for i in 0..typewriter.current_line {
                        full_text.push_str(current_page[i]);
                        full_text.push('\n');
                    }
                    
                    // 添加当前正在显示的行
                    let display_text: String = chars[..=typewriter.current_char].iter().collect();
                    full_text.push_str(&display_text);
                    
                    // 更新文本
                    **text = full_text;
                    
                    typewriter.current_char += 1;
                    let char_delay = typewriter.char_delay;
                    typewriter.timer.set_duration(char_delay);
                } else {
                    // 当前行完成，移到下一行
                    typewriter.current_line += 1;
                    typewriter.current_char = 0;
                    let line_delay = typewriter.line_delay;
                    typewriter.timer.set_duration(line_delay);
                }
            }
        }
    }
}

// 渐暗效果系统
fn fade_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TextColor, &mut TypewriterText, &mut FadeEffect)>,
) {
    for (mut text, mut text_color, mut typewriter, mut fade) in query.iter_mut() {
        if fade.is_fading_out {
            fade.timer.tick(time.delta());
            
            // 计算渐出进度
            let progress = fade.timer.fraction();
            let alpha = 1.0 - progress;
            text_color.0.set_alpha(alpha);
            
            if fade.timer.just_finished() {
                // 渐出完成，清空文本并准备下一页
                fade.is_fading_out = false;
                fade.is_fading_in = true;
                fade.timer.reset();
                
                // 切换到下一页
                typewriter.current_page += 1;
                typewriter.current_line = 0;
                typewriter.current_char = 0;
                
                // 清空文本
                **text = String::new();
            }
        } else if fade.is_fading_in {
            fade.timer.tick(time.delta());
            
            // 计算渐入进度
            let progress = fade.timer.fraction();
            let alpha = progress;
            text_color.0.set_alpha(alpha);
            
            if fade.timer.just_finished() {
                // 渐入完成，开始新页面的打字效果
                fade.is_fading_in = false;
                text_color.0.set_alpha(1.0);
                
                // 重置打字机定时器
                let char_delay = typewriter.char_delay;
                typewriter.timer.set_duration(char_delay);
            }
        }
    }
}
// 主函数示例
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Typewriter Text Effect with Auto Pages".to_string(),
                resolution: (1400.0, 750.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(TypewriterPlugin)
        .run();
}
