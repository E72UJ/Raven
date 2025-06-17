use bevy::{
    color::palettes::basic::*, 
    ecs::relationship::RelatedSpawnerCommands, 
    prelude::*, 
    winit::WinitSettings
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(NavigationVisible(false)) // 初始状态为隐藏
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, keyboard_input_system))
        .run();
}

// 导航栏可见性资源
#[derive(Resource)]
struct NavigationVisible(bool);

// 导航栏标记组件
#[derive(Component)]
struct NavigationBar;

const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// 键盘输入系统：监听按键切换导航栏显示
fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut nav_visible: ResMut<NavigationVisible>,
    mut nav_query: Query<&mut Visibility, With<NavigationBar>>,
) {
    // 按下 F4 键切换导航栏显示
    if keyboard_input.just_pressed(KeyCode::F4) {
        nav_visible.0 = !nav_visible.0;
        
        // 更新导航栏的可见性
        for mut visibility in nav_query.iter_mut() {
            *visibility = if nav_visible.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        
        println!("导航栏 {}", if nav_visible.0 { "显示" } else { "隐藏" });
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
    commands.spawn(Camera2d);
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(BLACK.into()),
    )).with_children(|parent| {
        // 主内容区域（90%高度）
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(95.0),
                ..default()
            },
            BackgroundColor(RED.into()),
        ));
        
        // 底部导航栏（10%高度）
        parent.spawn((
            NavigationBar, // 添加导航栏标记
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(8.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BLUE.into()),
            Visibility::Hidden, // 初始状态为隐藏
        )).with_children(|parent| {
            // 创建导航按钮
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