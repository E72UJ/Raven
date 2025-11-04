use bevy::prelude::*;
use bevy::window::WindowResolution;
use crate::raven::script::Script;
use bevy::app::AppExit; 
use crate::raven::scene::SceneCommand;

#[derive(Resource)]
pub struct RavenStory {
    pub story: Script,
    pub current_scene: Option<String>,
    pub scene_index: usize,
    pub waiting_for_input: bool,
}

#[derive(Component)]
pub struct CharacterSprite {
    pub character_id: String,
}

#[derive(Component)]
pub struct BackgroundSprite;

#[derive(Component)]
pub struct DialogueUI;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct SpeakerNameText;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

pub struct RavenPlugin;

impl Plugin for RavenPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .add_systems(Startup, setup_raven_game)
            .add_systems(
                Update,
                (
                    handle_input,
                    handle_scene_progress.after(handle_input),
                    update_dialogue_display.after(handle_scene_progress),
                )
                .run_if(in_state(GameState::Playing))
            );
    }
}

fn setup_raven_game(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Px(200.0),
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Px(20.0),
                            bottom: Val::Px(20.0),
                        },
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,    
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    BorderColor::all(Color::WHITE),
                    DialogueUI,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font: _asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SpeakerNameText,
                    ));

                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font: _asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            
                            ..default()
                        },
                        DialogueText,
                    ));
                });
        });
}

fn handle_input(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut raven_story: ResMut<RavenStory>) {
    if raven_story.waiting_for_input {
        if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
            raven_story.waiting_for_input = false;
            println!("用户点击，继续场景");
        }
    }
}

fn handle_scene_progress(mut raven_story: ResMut<RavenStory>, mut commands: Commands, asset_server: Res<AssetServer>, background_query: Query<Entity, With<BackgroundSprite>>, character_query: Query<(Entity, &CharacterSprite)>, mut exit: EventWriter<AppExit>, ) {
    if raven_story.waiting_for_input {
        return;
    }

    let current_scene_id = match &raven_story.current_scene {
        Some(id) => id.clone(),
        None => return,
    };

    let scene_commands = match raven_story.story.get_scene(&current_scene_id) {
        Some(scene) => scene.commands.clone(),
        None => return,
    };

    while raven_story.scene_index < scene_commands.len() && !raven_story.waiting_for_input {
        let command = scene_commands[raven_story.scene_index].clone();
        let should_pause = execute_simple_command(&command, &mut commands, &asset_server, &mut raven_story, &background_query, &character_query,&mut exit);

        raven_story.scene_index += 1;

        if should_pause {
            raven_story.waiting_for_input = true;
            break;
        }
    }

    if raven_story.scene_index >= scene_commands.len() && !raven_story.waiting_for_input {
        // println!("场景已结束");
    }
}

fn execute_simple_command(
    command: &SceneCommand, 
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>, 
    raven_story: &mut ResMut<RavenStory>, 
    background_query: &Query<Entity, With<BackgroundSprite>>, 
    character_query: &Query<(Entity, &CharacterSprite)>,
    exit: &mut EventWriter<AppExit>,
) -> bool {
    match command {
        SceneCommand::PlayMusic { file } => {
            println!("播放音乐: {}", file);
            false
        },
        SceneCommand::ShowBackground { background } => {
            for entity in background_query.iter() {
                commands.entity(entity).despawn();
            }

            if let Some(bg) = raven_story.story.get_background(background) {
                commands.spawn((
                    Sprite::from_image(asset_server.load(&bg.image)),
                    Transform::from_translation(Vec3::new(0.0, 0.0, -10.0))
                        .with_scale(Vec3::splat(1.5)),
                    BackgroundSprite,
                ));
                println!("显示背景: {}", background);
            }
            false
        },
        SceneCommand::ShowCharacter { character, emotion } => {
            let mut character_exists = false;
            for (_, char_comp) in character_query.iter() {
                if char_comp.character_id == *character {
                    character_exists = true;
                    break;
                }
            }

            if !character_exists {
                if let Some(char) = raven_story.story.get_character(character) {
                    commands.spawn((
                        Sprite::from_image(asset_server.load(&char.sprite)),
                        Transform::from_translation(Vec3::new(0.0, -100.0, 0.0))
                            .with_scale(Vec3::splat(0.75)),
                        CharacterSprite {
                            character_id: character.clone(),
                        },
                    ));

                    let emotion_text = emotion.as_ref().map(|e| format!(" [{}]", e)).unwrap_or_default();
                    println!("显示角色: {}{}", char.name, emotion_text);
                }
            }
            false
        },
        SceneCommand::HideCharacter { character } => {
            for (entity, char_comp) in character_query.iter() {
                if char_comp.character_id == *character {
                    commands.entity(entity).despawn();
                    println!("隐藏角色: {}", character);
                    break;
                }
            }
            false
        },
        SceneCommand::Dialogue { speaker, text } => {
            println!("对话: {} - {}", speaker, text);
            true
        },
        SceneCommand::PlayerThinks { text } => {
            println!("玩家思考: {}", text);
            true
        },
        SceneCommand::PlayerSays { text } => {
            println!("玩家说话: {}", text);
            true
        },
        SceneCommand::ShowChoices { choices: _ } => {
            println!("显示选择菜单 (简化版本不处理选择)");
            true
        },
        SceneCommand::Jump { scene } => {
            raven_story.current_scene = Some(scene.clone());
            raven_story.scene_index = 0;
            println!("跳转到场景: {}", scene);
            false
        },
        SceneCommand::EndWith { ending } => {
            println!("游戏结束: {}", ending);
            true
        },
        SceneCommand::ExitGame => {
            println!("退出游戏");
            exit.write(AppExit::default()); 
            false
        },
    }
}

fn update_dialogue_display(raven_story: Res<RavenStory>, mut speaker_query: Query<&mut Text, (With<SpeakerNameText>, Without<DialogueText>)>, mut dialogue_query: Query<&mut Text, (With<DialogueText>, Without<SpeakerNameText>)>) {
    if let Some(scene_id) = &raven_story.current_scene {
        if let Some(scene) = raven_story.story.get_scene(scene_id) {
            if raven_story.scene_index > 0 && raven_story.scene_index <= scene.commands.len() {
                let command = &scene.commands[raven_story.scene_index - 1];

                match command {
                    SceneCommand::Dialogue { speaker, text } => {
                        if let Ok(mut speaker_text) = speaker_query.single_mut() {
                            if let Some(character) = raven_story.story.get_character(speaker) {
                                **speaker_text = character.name.clone();
                            } else {
                                **speaker_text = speaker.clone();
                            }
                        }

                        if let Ok(mut dialogue_text) = dialogue_query.single_mut() {
                            **dialogue_text = text.clone();
                        }
                    },
                    SceneCommand::PlayerThinks { text } => {
                        if let Ok(mut speaker_text) = speaker_query.single_mut() {
                            **speaker_text = "内心想法".to_string();
                        }
                        if let Ok(mut dialogue_text) = dialogue_query.single_mut() {
                            **dialogue_text = text.clone();
                        }
                    },
                    SceneCommand::PlayerSays { text } => {
                        if let Ok(mut speaker_text) = speaker_query.single_mut() {
                            **speaker_text = "玩家".to_string();
                        }
                        if let Ok(mut dialogue_text) = dialogue_query.single_mut() {
                            **dialogue_text = text.clone();
                        }
                    },
                    _ => {
                        if let Ok(mut speaker_text) = speaker_query.single_mut() {
                            **speaker_text = "".to_string();
                        }
                        if let Ok(mut dialogue_text) = dialogue_query.single_mut() {
                            **dialogue_text = "".to_string();
                        }
                    }
                }
            }
        }
    }
}

pub fn run_raven_game(story_option: Option<Script>) {
    if let Some(story) = story_option {
        App::new()
            .add_plugins(
                
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Raven Visual Novel Engine".to_string(),
                        resolution: WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                })
                
            )
            .add_plugins(RavenPlugin)
            .insert_resource(RavenStory {
                current_scene: story.start_scene.clone(),
                story,
                scene_index: 0,
                waiting_for_input: false,
            })
            .insert_state(GameState::Playing)
            .run();
    } else {
        println!("没有提供故事内容！");
    }
}

pub fn get_game_result() -> String {
    "游戏已完成".to_string()
}

pub fn end_raven_game() {
    println!("清理游戏资源...");
}
