use bevy::prelude::*;
use bevy::window::{WindowMode, PrimaryWindow, MonitorSelection};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, handle_fullscreen_button)
        .run();
}

// 标记全屏按钮的组件
#[derive(Component)]
struct FullscreenButton;

fn setup_ui(mut commands: Commands) {
    // 创建相机
    commands.spawn(Camera2d);

    // 创建 UI 根节点
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 创建全屏按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                    FullscreenButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Toggle Fullscreen"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                    ));
                });
        });
}

fn handle_fullscreen_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<FullscreenButton>),
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // 按钮被点击时切换全屏状态
                if let Ok(mut window) = window_query.single_mut() {
                    window.mode = match window.mode {
                        WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                        WindowMode::BorderlessFullscreen(_) => WindowMode::Windowed,
                        WindowMode::Fullscreen(_, _) => WindowMode::Windowed,
                    };
                }
                *color = BackgroundColor(Color::srgb(0.1, 0.1, 0.6));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.9));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.8));
            }
        }
    }
}
