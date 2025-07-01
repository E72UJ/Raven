use bevy::ecs::system::command;
use bevy::prelude::*;

#[derive(Component)]
struct TypewriterText {
    full_text: String,
    current_length: usize,
    timer: Timer,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, typewriter_system)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 生成摄像机
    commands.spawn(Camera2d);

    // 生成带边框的精灵
    commands.spawn((
        Name::new("bordered_sprite"),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(50.0),
                left: Val::Px(50.0),
                right: Val::Px(50.0),
                height: Val::Px(170.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("textbox"),
                Text::new(""),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TypewriterText {
                    full_text: "hello hello hello".to_string(),
                    current_length: 0,
                    timer: Timer::from_seconds(0.05, TimerMode::Repeating), // 每50毫秒显示一个字符
                },
            ));
        });
}

fn typewriter_system(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut TypewriterText)>,
) {
    for (mut text, mut typewriter) in query.iter_mut() {
        typewriter.timer.tick(time.delta());
        
        if typewriter.timer.just_finished() && typewriter.current_length < typewriter.full_text.len() {
            typewriter.current_length += 1;
            text.0 = typewriter.full_text.chars().take(typewriter.current_length).collect();
        }
    }
}