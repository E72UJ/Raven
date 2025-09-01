use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_button_interaction)
        .run();
}

#[derive(Component)]
struct ButtonImages {
    normal: Handle<Image>,
    hovered: Handle<Image>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 摄像机
    commands.spawn(Camera2d);

    // 加载图片资源
    let normal_image = asset_server.load("gui/choice_idle_background2.png");
    let hovered_image = asset_server.load("gui/choice_hover_background2.png");

    // 创建UI容器
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 创建按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(790.0),
                    height: Val::Px(35.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ButtonImages {
                    normal: normal_image.clone(),
                    hovered: hovered_image,
                },
                ImageNode::new(normal_image),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|parent| {
                // 按钮文本
                parent.spawn((
                    Text::new("testone "),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

    println!("✅ 按钮已创建，移动鼠标到按钮上测试悬停效果");
}

fn handle_button_interaction(
    mut query: Query<
        (&Interaction, &ButtonImages, &mut ImageNode),
        (Changed<Interaction>, With<Button>)
    >,
) {
    for (interaction, button_images, mut image_node) in &mut query {
        match *interaction {
            Interaction::None => {
                image_node.image = button_images.normal.clone();
                println!("🔵 按钮状态: 正常");
            },
            Interaction::Hovered => {
                image_node.image = button_images.hovered.clone();
                println!("🟡 按钮状态: 悬停");
            },
            Interaction::Pressed => {
                image_node.image = button_images.normal.clone();
                println!("🔴 按钮状态: 按下");
            },
        }
    }
}
