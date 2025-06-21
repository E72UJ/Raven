use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_ui)
        .run();
}

#[derive(Resource)]
struct UiElement {
    entity: Entity,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 创建相机
    commands.spawn(Camera2d);

    // 创建带纹理的文本框容器
    let textbox_container = commands
        .spawn(Node {
            width: Val::Px(400.0),
            height: Val::Px(200.0),
            position_type: PositionType::Absolute,
            left: Val::Px(100.0),
            top: Val::Px(100.0),
            border: UiRect::all(Val::Px(2.0)),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.8)))
        .insert(BorderColor(Color::WHITE))
        .with_children(|parent| {
            // 文本内容
            parent.spawn((
                Text::new("这是一个带纹理的文本框！\n可以显示多行文本。"),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
            ));
        })
        .id();

    // 存储UI元素实体
    commands.insert_resource(UiElement { entity: textbox_container });
}

fn update_ui(
    time: Res<Time>,
    ui_element: Res<UiElement>,
    mut query: Query<&mut Node>,
) {
    if let Ok(mut node) = query.get_mut(ui_element.entity) {
        // 让文本框左右移动
        let offset = (time.elapsed_secs() * 2.0).sin() * 50.0;
        node.left = Val::Px(100.0 + offset);
    }
}