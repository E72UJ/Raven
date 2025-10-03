use bevy::{
    color::palettes::basic::*,
    ecs::relationship::RelatedSpawnerCommands,
    prelude::*,
    winit::WinitSettings,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings::desktop_app())
        // 添加导航栏可见性状态资源
        .insert_resource(NavBarVisible(false)) // 默认隐藏
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, toggle_button_system))
        .run();
}

// 导航栏可见性状态资源
#[derive(Resource)]
struct NavBarVisible(bool);

// 为导航栏添加标记组件
#[derive(Component)]
struct NavBar;

// 为切换按钮添加标记组件
#[derive(Component)]
struct ToggleButton;

const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// 处理切换按钮的系统
fn toggle_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>, With<ToggleButton>)
    >,
    mut nav_visible: ResMut<NavBarVisible>,
    mut nav_query: Query<&mut Node, With<NavBar>>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                
                // 切换导航栏可见性
                nav_visible.0 = !nav_visible.0;
                
                // 更新导航栏的高度
                if let Ok(mut nav_node) = nav_query.single_mut() {
                    nav_node.height = if nav_visible.0 {
                        Val::Px(50.0) // 显示时的高度
                    } else {
                        Val::Px(0.0)  // 隐藏时高度为0
                    };
                }
                
                println!("导航栏{}", if nav_visible.0 { "显示" } else { "隐藏" });
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

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
            &Name,
        ),
        (Changed<Interaction>, With<Button>, Without<ToggleButton>), // 排除切换按钮
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, nav_visible: Res<NavBarVisible>) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|parent| {
            // 主内容区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0), // 占满剩余空间
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|content_parent| {
                    // 添加切换按钮在右下角
                    content_parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                right: Val::Px(20.0),
                                bottom: Val::Px(20.0),
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                border: UiRect::all(Val::Px(2.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            Button,
                            ToggleButton, // 标记为切换按钮
                            BorderColor(Color::BLACK),
                            BorderRadius::all(Val::Px(30.0)), // 圆形按钮
                            BackgroundColor(NORMAL_BUTTON),
                        ))
                        .with_child((
                            Text::new("☰"), // 使用汉堡菜单图标
                            TextFont {
                                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                });

            // 底部导航栏
            parent
                .spawn((
                    NavBar, // 标记为导航栏
                    Node {
                        width: Val::Percent(100.0),
                        height: if nav_visible.0 { Val::Px(50.0) } else { Val::Px(0.0) }, // 根据状态设置高度
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(), // 隐藏溢出内容
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                ))
                .with_children(|parent| {
                    let nav_items = vec!["主菜单", "保存", "读取", "设置", "历史", "跳过", "自动"];

                    for item in nav_items {
                        create_nav_button(parent, &asset_server, item);
                    }
                });
        });
}

fn create_nav_button(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    label: &str,
) {
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