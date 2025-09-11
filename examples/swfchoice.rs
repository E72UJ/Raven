use bevy::prelude::*;
use std::path::PathBuf;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SwfPlayerState>()
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (
            button_interaction_system,
            handle_file_dialog,
            update_file_display,
        ))
        .run();
}

#[derive(Resource, Default)]
struct SwfPlayerState {
    selected_file: Option<PathBuf>,
    show_dialog: bool,
}

#[derive(Component)]
struct SelectButton;

#[derive(Component)]
struct SwfDisplayArea;

#[derive(Component)]
struct FileDialog;

#[derive(Component)]
struct FilePathText;

#[derive(Component)]
struct DialogConfirmButton;

#[derive(Component)]
struct DialogCancelButton;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 相机
    commands.spawn(Camera2d);

    // 主UI容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .with_children(|parent| {
            // 顶部控制区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // 选择文件按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.8)),
                            SelectButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Select SWF File"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 显示选中文件路径
                    parent.spawn((
                        Text::new("No file selected"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        FilePathText,
                    ));
                });

            // SWF显示区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                    BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                    SwfDisplayArea,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("SWF Player Area\nSelect a file to display here"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });
        });
}

fn button_interaction_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>)
    >,
    select_button_query: Query<Entity, With<SelectButton>>,
    confirm_button_query: Query<Entity, With<DialogConfirmButton>>,
    cancel_button_query: Query<Entity, With<DialogCancelButton>>,
    mut swf_state: ResMut<SwfPlayerState>,
    mut commands: Commands,
    dialog_query: Query<Entity, With<FileDialog>>,
) {
    for (entity, interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                color.0 = Color::srgb(0.2, 0.4, 0.7);
                
                // 处理选择按钮
                if select_button_query.contains(entity) {
                    if !swf_state.show_dialog {
                        swf_state.show_dialog = true;
                        spawn_file_dialog(&mut commands);
                    }
                }
                
                // 处理确认按钮
                if confirm_button_query.contains(entity) {
                    swf_state.selected_file = Some(PathBuf::from("example.swf"));
                    close_dialog(&mut commands, &mut swf_state, &dialog_query);
                }
                
                // 处理取消按钮
                if cancel_button_query.contains(entity) {
                    close_dialog(&mut commands, &mut swf_state, &dialog_query);
                }
            }
            Interaction::Hovered => {
                color.0 = Color::srgb(0.35, 0.55, 0.85);
            }
            Interaction::None => {
                color.0 = Color::srgb(0.3, 0.5, 0.8);
            }
        }
    }
}

fn spawn_file_dialog(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                top: Val::Percent(30.0),
                width: Val::Percent(50.0),
                height: Val::Px(200.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
            FileDialog,
        ))
        .with_children(|parent| {
            // 对话框标题
            parent.spawn((
                Text::new("Select SWF File"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 输入框区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(40.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Enter SWF file path..."),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });

            // 按钮区域
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // 确认按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                            DialogConfirmButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Confirm"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 取消按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
                            DialogCancelButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Cancel"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn handle_file_dialog(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut swf_state: ResMut<SwfPlayerState>,
    dialog_query: Query<Entity, With<FileDialog>>,
) {
    if swf_state.show_dialog && keyboard_input.just_pressed(KeyCode::Escape) {
        close_dialog(&mut commands, &mut swf_state, &dialog_query);
    }
}

fn close_dialog(
    commands: &mut Commands,
    swf_state: &mut ResMut<SwfPlayerState>,
    dialog_query: &Query<Entity, With<FileDialog>>,
) {
    if let Ok(dialog_entity) = dialog_query.get_single() {
        commands.entity(dialog_entity).despawn_recursive();
    }
    swf_state.show_dialog = false;
}

fn update_file_display(
    swf_state: Res<SwfPlayerState>,
    mut text_query: Query<&mut Text, With<FilePathText>>,
    mut display_area_query: Query<&mut BackgroundColor, With<SwfDisplayArea>>,
) {
    if swf_state.is_changed() {
        if let Ok(mut text) = text_query.get_single_mut() {
            if let Some(file_path) = &swf_state.selected_file {
                **text = format!("Selected: {}", file_path.display());
                
                if let Ok(mut bg_color) = display_area_query.get_single_mut() {
                    bg_color.0 = Color::srgb(0.1, 0.2, 0.1);
                }
            } else {
                **text = "No file selected".to_string();
            }
        }
    }
}