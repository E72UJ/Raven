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
        .init_state::<AppState>() // 添加状态管理
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, handle_save_popup, close_popup_system))
        .run();
}

// 定义应用状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Normal,
    SavePopup, // 存档弹窗状态
}

// 组件标记
#[derive(Component)]
struct SavePopup; // 存档弹窗标记

#[derive(Component)]
struct SaveSlot(usize); // 存档槽位标记

#[derive(Component)]
struct PopupOverlay; // 弹窗遮罩层

const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

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
    mut next_state: ResMut<NextState<AppState>>, // 添加状态控制
    current_state: Res<State<AppState>>,
) {
    for (interaction, mut color, mut border_color, children, name) in &mut interaction_query {
        let _text = text_query.get_mut(children[0]).unwrap();
        
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                println!("按下了: {}", name.as_str());
                
                // 检查是否点击了保存按钮
                if name.as_str() == "保存" && *current_state.get() == AppState::Normal {
                    next_state.set(AppState::SavePopup);
                }
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

// 处理存档弹窗的显示和隐藏
fn handle_save_popup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_state: Res<State<AppState>>,
    popup_query: Query<Entity, With<SavePopup>>,
) {
    match current_state.get() {
        AppState::SavePopup => {
            // 如果弹窗不存在，创建弹窗
            if popup_query.is_empty() {
                create_save_popup(&mut commands, &asset_server);
            }
        }
        AppState::Normal => {
            // 如果弹窗存在，删除弹窗
            for entity in popup_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

// 创建存档弹窗
fn create_save_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            // 遮罩层：半透明黑色背景
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            SavePopup,
            PopupOverlay,
        ))
        .with_children(|parent| {
            // 弹窗主体
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                    BorderColor(Color::srgb(0.5, 0.5, 0.6)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    // 标题
                    popup.spawn((
                        Text::new("存档管理"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // 存档槽位容器
                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(250.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|slots_container| {
                            // 创建5个存档槽位
                            for i in 1..=5 {
                                create_save_slot(slots_container, asset_server, i);
                            }
                        });

                    // 关闭按钮
                    popup
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(20.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                            BorderColor(Color::BLACK),
                            BorderRadius::all(Val::Px(5.0)),
                            Name::new("关闭".to_string()),
                        ))
                        .with_child((
                            Text::new("关闭"),
                            TextFont {
                                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                });
        });
}

// 创建存档槽位
fn create_save_slot(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    slot_number: usize,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
            BorderColor(Color::srgb(0.4, 0.4, 0.5)),
            BorderRadius::all(Val::Px(5.0)),
            SaveSlot(slot_number),
            Name::new(format!("存档槽位{}", slot_number)),
        ))
        .with_children(|slot| {
            // 槽位信息文本
            slot.spawn((
                Text::new(format!("存档槽位 {} - 空", slot_number)),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // 时间信息文本
            slot.spawn((
                Text::new("--/--/-- --:--"),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

// 处理弹窗关闭
fn close_popup_system(
    mut interaction_query: Query<
        (&Interaction, &Name),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // 检查是否点击了关闭按钮
    for (interaction, name) in &mut interaction_query {
        if *interaction == Interaction::Pressed && name.as_str() == "关闭" {
            next_state.set(AppState::Normal);
        }
    }

    // 按ESC键关闭弹窗
    if keyboard_input.just_pressed(KeyCode::Escape) && *current_state.get() == AppState::SavePopup {
        next_state.set(AppState::Normal);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(90.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

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