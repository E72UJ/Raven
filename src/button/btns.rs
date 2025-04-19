use bevy::{color::palettes::basic::*, prelude::*, winit::WinitSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // 只在有用户输入时运行应用，以减少CPU/GPU使用
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor, &Children, &Name),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children, name) in &mut interaction_query {
        let _text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                println!("按下了: {}", name.as_str());
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI相机
    commands.spawn(Camera2d);
    
    // 主容器
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|parent| {
            // 主内容区域（这里可以放其他内容）
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
            
            // 底部导航栏
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(10.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|parent| {
                    // 创建所有导航按钮
                    let nav_items = vec![
                        "主菜单", "保存", "读取", "设置", "历史", "跳过", "自动"
                    ];
                    
                    for item in nav_items {
                        create_nav_button(parent, &asset_server, item);
                    }
                });
        });
}

fn create_nav_button(parent: &mut ChildBuilder, asset_server: &Res<AssetServer>, label: &str) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(80.0),
                height: Val::Px(40.0),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::all(Val::Px(5.0)),
            BackgroundColor(NORMAL_BUTTON),
            Name::new(label.to_string()),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
}