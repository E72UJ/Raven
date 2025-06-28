use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// 数据结构定义
#[derive(Deserialize, Serialize, Debug, Clone)]
struct DialogueChoice {
    text: String,
    goto: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DialogueEntry {
    character: String,
    text: String,
    portrait: String,
    choices: Option<Vec<DialogueChoice>>,
}

// 组件定义
#[derive(Component)]
struct ChoiceButton {
    goto_index: usize,
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct CharacterName;

#[derive(Component)]
struct ButtonContainer;

// 资源定义
#[derive(Resource)]
struct DialogueData {
    entries: Vec<DialogueEntry>,
}

#[derive(Resource)]
struct CurrentDialogue {
    index: usize,
}

#[derive(Resource)]
struct DialogueFonts {
    regular: Handle<Font>,
}

// 字体样式常量
const BUTTON_FONT_SIZE: f32 = 16.0;
const DIALOGUE_FONT_SIZE: f32 = 18.0;
const CHARACTER_NAME_FONT_SIZE: f32 = 24.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DialogueData {
            entries: load_dialogue_data(),
        })
        .insert_resource(CurrentDialogue { index: 0 })
        .add_systems(Startup, (load_fonts, setup_ui).chain())
        .add_systems(Update, (update_dialogue_display, handle_choice_buttons))
        .run();
}

// 加载字体
fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let regular_font = asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf");
    
    commands.insert_resource(DialogueFonts {
        regular: regular_font,
    });
}

// 加载对话数据
fn load_dialogue_data() -> Vec<DialogueEntry> {
    vec![
        DialogueEntry {
            character: "黛安娜".to_string(),
            text: "你好，欢迎使用Raven Engine V0.1".to_string(),
            portrait: "protagonist".to_string(),
            choices: None,
        },
        DialogueEntry {
            character: "黛安娜".to_string(),
            text: "第二句".to_string(),
            portrait: "protagonist".to_string(),
            choices: Some(vec![
                DialogueChoice {
                    text: "选择一".to_string(),
                    goto: 2,
                },
                DialogueChoice {
                    text: "选择二".to_string(),
                    goto: 2,
                },
                DialogueChoice {
                    text: "选择三".to_string(),
                    goto: 0,
                },
                DialogueChoice {
                    text: "选择四".to_string(),
                    goto: 1,
                },
            ]),
        },
        DialogueEntry {
            character: "黛安娜".to_string(),
            text: "第三句".to_string(),
            portrait: "protagonist".to_string(),
            choices: None,
        },
    ]
}

// 辅助函数创建统一样式的文本
fn create_text_bundle(
    text: String,
    font: Handle<Font>,
    font_size: f32,
    color: Color,
) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(color),
    )
}

// 初始化 UI
fn setup_ui(mut commands: Commands, fonts: Res<DialogueFonts>) {
    // 摄像机
    commands.spawn(Camera2d);

    // 主容器
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        })
        .insert(BackgroundColor(Color::srgb(0.1, 0.1, 0.15)))
        .with_children(|parent| {
            // 对话框容器
            parent
                .spawn(Node {
                    width: Val::Px(800.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .insert(BackgroundColor(Color::srgb(0.2, 0.2, 0.25)))
                .with_children(|dialog_parent| {
                    // 角色名字
                    dialog_parent.spawn((
                        create_text_bundle(
                            "".to_string(),
                            fonts.regular.clone(),
                            CHARACTER_NAME_FONT_SIZE,
                            Color::srgb(0.9, 0.9, 0.9),
                        ),
                        CharacterName,
                    ));

                    // 对话文本
                    dialog_parent.spawn((
                        create_text_bundle(
                            "".to_string(),
                            fonts.regular.clone(),
                            DIALOGUE_FONT_SIZE,
                            Color::WHITE,
                        ),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                        DialogueText,
                    ));
                });

            // 按钮容器
            parent.spawn((
                Node {
                    width: Val::Px(800.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                ButtonContainer,
            ));
        });
}

// 更新对话显示
fn update_dialogue_display(
    dialogue_data: Res<DialogueData>,
    current_dialogue: Res<CurrentDialogue>,
    fonts: Res<DialogueFonts>,
    mut character_query: Query<&mut Text, (With<CharacterName>, Without<DialogueText>)>,
    mut dialogue_query: Query<&mut Text, (With<DialogueText>, Without<CharacterName>)>,
    mut commands: Commands,
    button_container_query: Query<Entity, With<ButtonContainer>>,
    existing_buttons: Query<Entity, (With<ChoiceButton>, With<Button>)>,
) {
    if !current_dialogue.is_changed() {
        return;
    }

    // 获取当前对话条目
    if let Some(entry) = dialogue_data.entries.get(current_dialogue.index) {
        // 更新角色名字
        if let Ok(mut character_text) = character_query.get_single_mut() {
            character_text.0 = entry.character.clone();
        }

        // 更新对话文本
        if let Ok(mut dialogue_text) = dialogue_query.get_single_mut() {
            dialogue_text.0 = entry.text.clone();
        }

        // 清除现有按钮
        for entity in existing_buttons.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // 创建新按钮
        if let Some(choices) = &entry.choices {
            if let Ok(container_entity) = button_container_query.get_single() {
                commands.entity(container_entity).with_children(|parent| {
                    for choice in choices {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(400.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(5.0)),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                                BorderColor(Color::srgb(0.5, 0.5, 0.6)),
                                ChoiceButton {
                                    goto_index: choice.goto,
                                },
                            ))
                            .with_children(|button| {
                                button.spawn(create_text_bundle(
                                    choice.text.clone(),
                                    fonts.regular.clone(),
                                    BUTTON_FONT_SIZE,
                                    Color::WHITE,
                                ));
                            });
                    }
                });
            }
        } else {
            // 没有选择时，添加继续按钮
            if let Ok(container_entity) = button_container_query.get_single() {
                commands.entity(container_entity).with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(200.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
                            BorderColor(Color::srgb(0.6, 0.6, 0.7)),
                            ChoiceButton {
                                goto_index: (current_dialogue.index + 1) % dialogue_data.entries.len(),
                            },
                        ))
                        .with_children(|button| {
                            button.spawn(create_text_bundle(
                                "继续".to_string(),
                                fonts.regular.clone(),
                                BUTTON_FONT_SIZE,
                                Color::WHITE,
                            ));
                        });
                });
            }
        }
    }
}

// 处理按钮点击
fn handle_choice_buttons(
    mut interaction_query: Query<
        (&Interaction, &ChoiceButton, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut current_dialogue: ResMut<CurrentDialogue>,
) {
    for (interaction, choice_button, mut bg_color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                current_dialogue.index = choice_button.goto_index;
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.3));
                *border_color = BorderColor(Color::srgb(0.4, 0.4, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.5));
                *border_color = BorderColor(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
                *border_color = BorderColor(Color::srgb(0.5, 0.5, 0.6));
            }
        }
    }
}