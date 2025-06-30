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
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, handle_popups, close_popup_system))
        .run();
}

// 扩展应用状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Normal,
    MainMenu,    // 主菜单
    SavePopup,   // 存档弹窗
    LoadPopup,   // 读取弹窗
    Settings,    // 设置弹窗
    History,     // 历史弹窗
    SkipMode,    // 跳过模式
    AutoMode,    // 自动模式
}

// 组件标记
#[derive(Component)]
struct Popup; // 通用弹窗标记

#[derive(Component)]
struct SaveSlot(usize);

#[derive(Component)]
struct HistoryEntry(usize);

#[derive(Component)]
struct SettingSlider;

#[derive(Component)]
struct PopupOverlay;

// const NORMAL_BUTTON: Color = Color::srgb(0.75, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const NORMAL_BUTTON: Color = Color::srgba(0.1, 0.1, 0.1, 0.8);
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
    mut next_state: ResMut<NextState<AppState>>,
    current_state: Res<State<AppState>>,
) {
    for (interaction, mut color, mut border_color, children, name) in &mut interaction_query {
        let _text = text_query.get_mut(children[0]).unwrap();
        
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                println!("按下了: {}", name.as_str());
                
                // 只在正常状态下响应导航按钮
                if *current_state.get() == AppState::Normal {
                    match name.as_str() {
                        "主菜单" => next_state.set(AppState::MainMenu),
                        "保存" => next_state.set(AppState::SavePopup),
                        "读取" => next_state.set(AppState::LoadPopup),
                        "设置" => next_state.set(AppState::Settings),
                        "历史" => next_state.set(AppState::History),
                        "跳过" => next_state.set(AppState::SkipMode),
                        "自动" => next_state.set(AppState::AutoMode),
                        _ => {}
                    }
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

// 处理所有弹窗的显示和隐藏
fn handle_popups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_state: Res<State<AppState>>,
    popup_query: Query<Entity, With<Popup>>,
) {
    let should_show_popup = !matches!(current_state.get(), AppState::Normal);
    
    if should_show_popup && popup_query.is_empty() {
        // 根据状态创建对应的弹窗
        match current_state.get() {
            AppState::MainMenu => create_main_menu_popup(&mut commands, &asset_server),
            AppState::SavePopup => create_save_popup(&mut commands, &asset_server),
            AppState::LoadPopup => create_load_popup(&mut commands, &asset_server),
            AppState::Settings => create_settings_popup(&mut commands, &asset_server),
            AppState::History => create_history_popup(&mut commands, &asset_server),
            AppState::SkipMode => create_skip_mode_popup(&mut commands, &asset_server),
            AppState::AutoMode => create_auto_mode_popup(&mut commands, &asset_server),
            _ => {}
        }
    } else if !should_show_popup {
        // 删除所有弹窗
        for entity in popup_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// 主菜单弹窗
fn create_main_menu_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                    BorderColor(Color::srgb(0.4, 0.4, 0.6)),
                    BorderRadius::all(Val::Px(15.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("主菜单"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    let menu_items = vec!["新游戏", "继续游戏", "章节选择", "画廊", "音乐鉴赏", "退出游戏"];
                    
                    for item in menu_items {
                        create_popup_button(popup, asset_server, item, Val::Px(250.0), Val::Px(50.0));
                    }

                    create_popup_button(popup, asset_server, "返回", Val::Px(120.0), Val::Px(40.0));
                });
        });
}

// 读取存档弹窗
fn create_load_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(650.0),
                        height: Val::Px(450.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.3, 0.2)),
                    BorderColor(Color::srgb(0.4, 0.6, 0.4)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("读取存档"),
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

                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(300.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|slots_container| {
                            for i in 1..=6 {
                                create_load_slot(slots_container, asset_server, i);
                            }
                        });

                    create_popup_button(popup, asset_server, "关闭", Val::Px(100.0), Val::Px(40.0));
                });
        });
}

// 设置弹窗
fn create_settings_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.0),
                        height: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.2, 0.3)),
                    BorderColor(Color::srgb(0.5, 0.4, 0.6)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("游戏设置"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 设置选项
                    let settings = vec![
                        ("主音量", "80%"),
                        ("音效音量", "100%"),
                        ("语音音量", "90%"),
                        ("文字速度", "中等"),
                        ("自动播放速度", "慢"),
                        ("全屏显示", "开启"),
                    ];

                    for (setting, value) in settings {
                        create_setting_row(popup, asset_server, setting, value);
                    }

                    // 按钮区域
                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceAround,
                            margin: UiRect::top(Val::Px(30.0)),
                            ..default()
                        })
                        .with_children(|buttons| {
                            create_popup_button(buttons, asset_server, "应用", Val::Px(80.0), Val::Px(40.0));
                            create_popup_button(buttons, asset_server, "重置", Val::Px(80.0), Val::Px(40.0));
                            create_popup_button(buttons, asset_server, "关闭", Val::Px(80.0), Val::Px(40.0));
                        });
                });
        });
}

// 历史记录弹窗
fn create_history_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(700.0),
                        height: Val::Px(500.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.25, 0.2)),
                    BorderColor(Color::srgb(0.6, 0.5, 0.4)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("对话历史"),
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

                    // 历史记录滚动区域
                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(350.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        })
                        .with_children(|history_area| {
                            let history_entries = vec![
                                ("小明", "你好，今天天气真不错呢！"),
                                ("小红", "是啊，我们出去走走吧。"),
                                ("小明", "好主意，去公园怎么样？"),
                                ("小红", "太好了，我正想去看看樱花。"),
                                ("旁白", "两人愉快地向公园走去..."),
                            ];

                            for (i, (speaker, text)) in history_entries.iter().enumerate() {
                                create_history_entry(history_area, asset_server, i, speaker, text);
                            }
                        });

                    create_popup_button(popup, asset_server, "关闭", Val::Px(100.0), Val::Px(40.0));
                });
        });
}

// 跳过模式弹窗
fn create_skip_mode_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.2, 0.2)),
                    BorderColor(Color::srgb(0.6, 0.4, 0.4)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("跳过模式"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    popup.spawn((
                        Text::new("选择跳过方式："),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    create_popup_button(popup, asset_server, "跳过已读文本", Val::Px(200.0), Val::Px(40.0));
                    create_popup_button(popup, asset_server, "跳过所有文本", Val::Px(200.0), Val::Px(40.0));
                    create_popup_button(popup, asset_server, "取消", Val::Px(100.0), Val::Px(40.0));
                });
        });
}

// 自动模式弹窗
fn create_auto_mode_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Popup,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(350.0),
                        height: Val::Px(250.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.3, 0.2)),
                    BorderColor(Color::srgb(0.4, 0.6, 0.4)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new("自动播放模式"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    popup.spawn((
                        Text::new("✓ 自动播放已启用\n\n点击屏幕任意位置可停止"),
                        TextFont {
                            font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 1.0, 0.8)),
                    ));

                    create_popup_button(popup, asset_server, "停止自动播放", Val::Px(150.0), Val::Px(40.0));
                });
        });
}

// 辅助函数：创建存档弹窗
fn create_save_popup(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Popup,
        ))
        .with_children(|parent| {
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

                    popup
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(250.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|slots_container| {
                            for i in 1..=5 {
                                create_save_slot(slots_container, asset_server, i);
                            }
                        });

                    create_popup_button(popup, asset_server, "关闭", Val::Px(100.0), Val::Px(40.0));
                });
        });
}

// 辅助函数：创建弹窗按钮
fn create_popup_button(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    label: &str,
    width: Val,
    height: Val,
) {
    parent
        .spawn((
            Button,
            Node {
                width,
                height,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
            BorderColor(Color::BLACK),
            BorderRadius::all(Val::Px(5.0)),
            Name::new(label.to_string()),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

// 辅助函数：创建存档槽位
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
            slot.spawn((
                Text::new(format!("存档槽位 {} - 空", slot_number)),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

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

// 辅助函数：创建读取槽位
fn create_load_slot(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    slot_number: usize,
) {
    let has_save = slot_number <= 3; // 前3个槽位有存档
    
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(45.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(if has_save { Color::srgb(0.2, 0.4, 0.2) } else { Color::srgb(0.3, 0.3, 0.3) }),
            BorderColor(Color::srgb(0.4, 0.4, 0.5)),
            BorderRadius::all(Val::Px(5.0)),
            Name::new(format!("读取槽位{}", slot_number)),
        ))
        .with_children(|slot| {
            if has_save {
                slot.spawn((
                    Text::new(format!("存档 {} - 第{}章 樱花飞舞", slot_number, slot_number)),
                    TextFont {
                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                slot.spawn((
                    Text::new("2024/01/15 14:32"),
                    TextFont {
                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            } else {
                slot.spawn((
                    Text::new(format!("槽位 {} - 空", slot_number)),
                    TextFont {
                        font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            }
        });
}

// 辅助函数：创建设置行
fn create_setting_row(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    setting_name: &str,
    current_value: &str,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(setting_name),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            row.spawn((
                Text::new(current_value),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 1.0, 0.8)),
            ));
        });
}

// 辅助函数：创建历史记录条目
fn create_history_entry(
    parent: &mut RelatedSpawnerCommands<'_, ChildOf>,
    asset_server: &Res<AssetServer>,
    index: usize,
    speaker: &str,
    text: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.4, 0.3, 0.2, 0.3)),
            BorderColor(Color::srgb(0.5, 0.4, 0.3)),
            BorderRadius::all(Val::Px(5.0)),
            HistoryEntry(index),
            Name::new(format!("历史记录{}", index)),
        ))
        .with_children(|entry| {
            entry.spawn((
                Text::new(speaker),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.6)),
            ));

            entry.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
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
    // 检查关闭按钮点击
    for (interaction, name) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match name.as_str() {
                "关闭" | "返回" | "取消" | "停止自动播放" => {
                    next_state.set(AppState::Normal);
                }
                _ => {}
            }
        }
    }

    // ESC键关闭弹窗
    if keyboard_input.just_pressed(KeyCode::Escape) && *current_state.get() != AppState::Normal {
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