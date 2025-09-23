use bevy::prelude::*;
const NORMAL_BUTTON_FONT: &str = "fonts/SarasaFixedHC-Light.ttf";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, button_interaction_system)
        .run();
}

#[derive(Component, Clone, Copy)]
enum VnButton {
    Rollback,
    History,
    Skip,
    Auto,
    Save,
    Load,
    Settings,
}

impl VnButton {
    fn text(&self) -> &'static str {
        match self {
            VnButton::Rollback => "回退",
            VnButton::History => "历史",
            VnButton::Skip => "快进",
            VnButton::Auto => "自动",
            VnButton::Save => "保存",
            VnButton::Load => "读档",
            VnButton::Settings => "设置",
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 摄像机
    commands.spawn(Camera2d);

    // 加载字体
    let font_handle = asset_server.load(NORMAL_BUTTON_FONT);
commands
    .spawn((
        Sprite {
            image: asset_server.load("background/bg2.png"),
            ..default()
        },
        Transform {
            scale: Vec3::new(1.0, 1.0, 1.0), // 放大两倍
            ..default()
        },
    ));

    // 创建底部按钮栏容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            // 创建按钮
            let buttons = [
                VnButton::Rollback,
                VnButton::History,
                VnButton::Skip,
                VnButton::Auto,
                VnButton::Save,
                VnButton::Load,
                VnButton::Settings,
            ];

            for button_type in buttons {
                parent
                    .spawn((
                        Button,
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE), // 透明背景
                        button_type,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(button_type.text()),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn button_interaction_system(
    mut button_query: Query<(&Interaction, &Children, &VnButton), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children, button_type) in &mut button_query {
        // 找到按钮的文字子组件并更新颜色
        for child in children.iter() {  // 改为 child 而不是 &child
            if let Ok(mut text_color) = text_query.get_mut(child) {
                match *interaction {
                    Interaction::Pressed => {
                        *text_color = TextColor(Color::srgba(0.3, 0.5, 1.0, 1.0)); // 深蓝色
                        
                        // 处理按钮点击事件
                        match button_type {
                            VnButton::Rollback => println!("回退按钮被点击"),
                            VnButton::History => println!("历史按钮被点击"),
                            VnButton::Skip => println!("快进按钮被点击"),
                            VnButton::Auto => println!("自动按钮被点击"),
                            VnButton::Save => println!("保存按钮被点击"),
                            VnButton::Load => println!("读档按钮被点击"),
                            VnButton::Settings => println!("设置按钮被点击"),
                        }
                    }
                    Interaction::Hovered => {
                        *text_color = TextColor(Color::srgba(0.4, 0.6, 1.0, 1.0)); // 蓝色
                    }
                    Interaction::None => {
                        *text_color = TextColor(Color::WHITE); // 白色
                    }
                }
            }
        }
    }
}
