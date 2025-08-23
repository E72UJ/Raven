use bevy::prelude::*;

// 定义按钮颜色 - 使用正确的颜色API
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct Option1Button;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_ui)
        .add_systems(Update, button_interaction_system)
        .run();
}

fn setup_ui(mut commands: Commands) {
    // UI摄像机
    commands.spawn(Camera2d);

    // 创建按钮
    commands.spawn((
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(220.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Text::new("选项1"),
        BorderColor(Color::srgb(0.0, 0.0, 0.0)), // 黑色
        BorderRadius::all(Val::Px(1.0)),
        BackgroundColor(NORMAL_BUTTON),
        Option1Button,
    ));
}

fn button_interaction_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&Option1Button>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color, option1_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = Color::srgb(1.0, 0.0, 0.0).into(); // 红色
                
                if option1_button.is_some() {
                    println!("选项1被点击了！");
                    // 添加你的按钮逻辑
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                *border_color = Color::srgb(1.0, 1.0, 1.0).into(); // 白色
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                *border_color = Color::srgb(0.0, 0.0, 0.0).into(); // 黑色
            }
        }
    }
}