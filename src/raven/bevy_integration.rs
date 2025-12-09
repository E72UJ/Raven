use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::ui::ContentSize;
use bevy::window::{WindowResolution, WindowResized};
use crate::raven::script::Script;
use crate::raven::config; 
use bevy::app::AppExit; 
use std::collections::HashMap;
use crate::raven::scene::SceneCommand;


//  打字机组件
#[derive(Component)]
pub struct TypewriterEffect {
    pub full_text: String,
    pub current_char_index: usize,
    pub timer: Timer,
    pub is_finished: bool,
    pub chars_per_second: f32,
}
impl TypewriterEffect {
    pub fn new(text: String, chars_per_second: f32) -> Self {
        let duration = if chars_per_second > 0.0 { 
            1.0 / chars_per_second 
        } else { 
            0.05 // 默认每秒20个字符
        };
        
        Self {
            full_text: text,
            current_char_index: 0,
            timer: Timer::from_seconds(duration, TimerMode::Repeating),
            is_finished: false,
            chars_per_second,
        }
    }

    pub fn start_new_text(&mut self, text: String) {
        self.full_text = text;
        self.current_char_index = 0;
        self.is_finished = false;
        self.timer.reset();
    }

    pub fn skip_to_end(&mut self) {
        self.current_char_index = self.full_text.chars().count();
        self.is_finished = true;
    }

    pub fn get_current_text(&self) -> String {
        self.full_text.chars().take(self.current_char_index).collect()
    }
}


// 打字机资源

// 打字机组件开始
#[derive(Resource)]
pub struct CanvasConfig {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct VirtualScreenScale {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

// === 可缩放UI组件 ===
#[derive(Component)]
pub struct ScalableUI {
    pub original_width: f32,
    pub original_height: f32,
    pub original_font_size: Option<f32>,
    pub original_margin: UiRect,
    pub original_padding: UiRect,
    pub original_border: UiRect,
    pub original_custom_size: Option<Vec2>, // 为精灵添加
}

impl ScalableUI {
    pub fn new() -> Self {
        Self {
            original_width: 0.0,
            original_height: 0.0,
            original_font_size: None,
            original_margin: UiRect::all(Val::Px(0.0)),
            original_padding: UiRect::all(Val::Px(0.0)),
            original_border: UiRect::all(Val::Px(0.0)),
            original_custom_size: None,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.original_width = width;
        self.original_height = height;
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.original_font_size = Some(font_size);
        self
    }

    pub fn with_margin(mut self, margin: UiRect) -> Self {
        self.original_margin = margin;
        self
    }

    pub fn with_padding(mut self, padding: UiRect) -> Self {
        self.original_padding = padding;
        self
    }

    pub fn with_border(mut self, border: UiRect) -> Self {
        self.original_border = border;
        self
    }

    pub fn with_sprite_size(mut self, size: Vec2) -> Self {
        self.original_custom_size = Some(size);
        self
    }
}

// === 原有资源和组件保持不变 ===
#[derive(Resource)]
pub struct RavenStory {
    pub story: Script,
    pub current_scene: Option<String>,
    pub scene_index: usize,
    pub waiting_for_input: bool,
    pub waiting_for_asset_load: bool,
    pub waiting_for_typewriter: bool, 
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

#[derive(Component)]
pub struct LoadingAsset {
    pub asset_handle: Handle<Image>,
    pub asset_path: String,
}

#[derive(Resource)]
pub struct AssetCache {
    pub cached_backgrounds: HashMap<String, Handle<Image>>,
    pub cached_characters: HashMap<String, Handle<Image>>,
}

impl Default for AssetCache {
    fn default() -> Self {
        Self {
            cached_backgrounds: HashMap::new(),
            cached_characters: HashMap::new(),
        }
    }
}

// === 更新后的插件 ===
pub struct RavenPlugin;

impl Plugin for RavenPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .init_resource::<AssetCache>()
            // 添加虚拟屏幕资源
            .insert_resource(CanvasConfig {
                width: 1920.0,  // 视觉小说常用分辨率
                height: 1080.0,
            })
            .insert_resource(VirtualScreenScale { 
                scale: 1.0,
                offset_x: 0.0,
                offset_y: 0.0,
            })
            .add_systems(Startup, setup_raven_game)
            .add_systems(
                Update,
                (
                    // 添加窗口缩放系统
                    window_resize_system,
                    preload_all_assets,
                    check_asset_loading.after(preload_all_assets),
                    handle_input,
                    handle_scene_progress.after(handle_input),
                    update_dialogue_display.after(handle_scene_progress),
                )
                .run_if(in_state(GameState::Playing))
            );
    }
}

// === 更新的设置函数 ===
fn setup_raven_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建2D摄像机
    commands.spawn(Camera2d);

    // 创建主UI容器（虚拟屏幕的根节点）
    commands
        .spawn((
            Node {
                width: Val::Px(1920.0), // 虚拟屏幕宽度
                height: Val::Px(1080.0), // 虚拟屏幕高度
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(100),
            ScalableUI::new().with_size(1920.0, 1080.0),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Px(250.0),
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Px(20.0),
                            bottom: Val::Px(40.0),
                        },
                        padding: UiRect::all(Val::Px(30.0)),
                        border: UiRect::all(Val::Px(3.0)),
                        flex_direction: FlexDirection::Column,    
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
                    BorderColor::all(Color::WHITE),
                    DialogueUI,
                    ScalableUI::new()
                        .with_size(1728.0, 250.0) // 90% of 1920
                        .with_margin(UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Px(20.0),
                            bottom: Val::Px(40.0),
                        })
                        .with_padding(UiRect::all(Val::Px(30.0)))
                        .with_border(UiRect::all(Val::Px(3.0))),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("namebox"),
                        Text::new(""),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 28.0,
                            ..default()
                        },
                        // TextColor(Color::srgb(1.0, 0.9, 0.4)), 
                        TextColor(Color::WHITE),
                        SpeakerNameText,
                        ScalableUI::new().with_font_size(28.0),
                    ));

                    // 对话文本
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font: asset_server.load("fonts/SarasaFixedHC-Regular.ttf"),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::top(Val::Px(15.0)),
                            ..default()
                        },
                        DialogueText,
                        ScalableUI::new()
                            .with_font_size(22.0)
                            .with_margin(UiRect::top(Val::Px(15.0))),
                        TypewriterEffect::new(String::new(), 30.0), 
                    ));
                });
        });
}

// === 窗口缩放系统 ===
fn window_resize_system(
    mut resize_events: EventReader<WindowResized>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
    mut ui_query: Query<(&mut Node, &ScalableUI), Without<Text>>,
    mut text_query: Query<(&mut TextFont, &ScalableUI), With<Text>>,
    mut sprite_query: Query<&mut Sprite, (With<ScalableUI>, Without<Node>)>,
    canvas_config: Res<CanvasConfig>,
    mut virtual_scale: ResMut<VirtualScreenScale>,
) {
    for event in resize_events.read() {
        let window_width = event.width;
        let window_height = event.height;
        let virtual_width = canvas_config.width;
        let virtual_height = canvas_config.height;

        // 计算等比缩放
        let scale_x = window_width / virtual_width;
        let scale_y = window_height / virtual_height;
        let scale = scale_x.min(scale_y);

        // 计算居中偏移
        let scaled_width = virtual_width * scale;
        let scaled_height = virtual_height * scale;
        let offset_x = (window_width - scaled_width) / 2.0;
        let offset_y = (window_height - scaled_height) / 2.0;

        // 更新缩放资源
        virtual_scale.scale = scale;
        virtual_scale.offset_x = offset_x;
        virtual_scale.offset_y = offset_y;

        // 更新摄像机（2D精灵缩放）
        let camera_scale = 1.0 / scale;
        for mut projection in camera_query.iter_mut() {
            if let Projection::Orthographic(ortho) = projection.as_mut() {
                ortho.scale = camera_scale;
            }
        }

        // 更新UI节点
for (mut node, scalable_ui) in ui_query.iter_mut() {
            if scalable_ui.original_width > 0.0 {
                node.width = Val::Px(scalable_ui.original_width * scale);
            }
            if scalable_ui.original_height > 0.0 {
                node.height = Val::Px(scalable_ui.original_height * scale);
            }

            node.margin = scale_ui_rect(&scalable_ui.original_margin, scale);
            node.padding = scale_ui_rect(&scalable_ui.original_padding, scale);
            node.border = scale_ui_rect(&scalable_ui.original_border, scale);

            // 为根UI容器设置偏移
            if scalable_ui.original_width == virtual_width && scalable_ui.original_height == virtual_height {
                node.left = Val::Px(offset_x);
                node.top = Val::Px(offset_y);
            }
        }

        // 更新文本大小
        for (mut text_font, scalable_ui) in text_query.iter_mut() {
            if let Some(original_font_size) = scalable_ui.original_font_size {
                text_font.font_size = original_font_size * scale;
            }
        }

        // 更新精灵大小（如果有ScalableUI组件）
        for mut sprite in sprite_query.iter_mut() {
            // 精灵通过摄像机缩放处理，这里不需要额外处理
            // 但如果需要特定的精灵缩放，可以在这里添加
        }

    }
}

// === 辅助函数 ===
fn scale_ui_rect(rect: &UiRect, scale: f32) -> UiRect {
    UiRect {
        left: scale_val(&rect.left, scale),
        right: scale_val(&rect.right, scale),
        top: scale_val(&rect.top, scale),
        bottom: scale_val(&rect.bottom, scale),
    }
}

fn scale_val(val: &Val, scale: f32) -> Val {
    match val {
        Val::Px(px) => Val::Px(*px * scale),
        Val::Percent(percent) => Val::Percent(*percent),
        _ => *val,
    }
}

// === 更新的命令执行函数 ===
fn execute_simple_command(
    command: &SceneCommand, 
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>, 
    raven_story: &mut ResMut<RavenStory>, 
    background_query: &Query<Entity, With<BackgroundSprite>>, 
    character_query: &Query<(Entity, &CharacterSprite)>,
    dialogue_ui_query: &Query<Entity, With<DialogueUI>>,
    exit: &mut EventWriter<AppExit>,
    asset_cache: &Res<AssetCache>,
) -> bool {
    match command {
        SceneCommand::ShowBackground { background } => {
            for entity in background_query.iter() {
                commands.entity(entity).despawn();
            }

            if let Some(bg) = raven_story.story.get_background(background) {
                let handle = if let Some(cached_handle) = asset_cache.cached_backgrounds.get(&bg.image) {
                    cached_handle.clone()
                } else {
                    asset_server.load(&bg.image)
                };
                
                commands.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(1920.0, 1080.0)), // 匹配虚拟分辨率
                        ..Sprite::from_image(handle)
                    },
                    Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
                    BackgroundSprite,
                    ScalableUI::new().with_sprite_size(Vec2::new(1920.0, 1080.0)),
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
                        Transform::from_translation(Vec3::new(0.0, -200.0, 0.0))
                            .with_scale(Vec3::splat(1.0)), // 基于虚拟分辨率调整
                        CharacterSprite {
                            character_id: character.clone(),
                        },
                        ScalableUI::new(), // 角色也支持缩放
                    ));

                    let emotion_text = emotion.as_ref().map(|e| format!(" [{}]", e)).unwrap_or_default();
                    println!("显示角色: {}{}", char.name, emotion_text);
                }
            }
            false
        },
        // ... 其他命令保持不变
        _ => {
            // 复制原有的其他命令处理逻辑
            match command {
                SceneCommand::PlayMusic { file } => {
                    println!("播放音乐: {}", file);
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
                    exit.write(AppExit::Success);
                    false
                },
                SceneCommand::HideBackground => {
                    for entity in background_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    println!("隐藏背景");
                    false
                },
                SceneCommand::HideDialogueBox => {
                    for entity in dialogue_ui_query.iter() {
                        commands.entity(entity).insert(Visibility::Hidden);
                    }
                    println!("隐藏对话框");
                    false
                },
                _ => false,
            }
        }
    }
}

// === 其余函数保持不变 ===
fn handle_input(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, mut raven_story: ResMut<RavenStory>) {
    if raven_story.waiting_for_input {
        if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
            raven_story.waiting_for_input = false;
        }
    }
}

fn handle_scene_progress(
    mut raven_story: ResMut<RavenStory>, 
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    background_query: Query<Entity, With<BackgroundSprite>>, 
    character_query: Query<(Entity, &CharacterSprite)>,
    dialogue_ui_query: Query<Entity, With<DialogueUI>>,
    mut exit: EventWriter<AppExit>, 
    asset_cache: Res<AssetCache>
) {
    if raven_story.waiting_for_input || raven_story.waiting_for_asset_load { 
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
        let should_pause = execute_simple_command(&command, &mut commands, &asset_server, &mut raven_story, &background_query, &character_query, &dialogue_ui_query, &mut exit, &asset_cache);

        raven_story.scene_index += 1;

        if should_pause {
            raven_story.waiting_for_input = true;
            break;
        }
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
                    primary_window: Some(config::window::build()),
                    ..default()
                })
            )
            .add_plugins(RavenPlugin)
            .insert_resource(RavenStory {
                current_scene: story.start_scene.clone(),
                story,
                scene_index: 0,
                waiting_for_input: false,
                waiting_for_asset_load: true,
                waiting_for_typewriter: true,
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

fn preload_all_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    raven_story: Res<RavenStory>,
    mut asset_cache: ResMut<AssetCache>,
) {
    for (_, background) in &raven_story.story.backgrounds {
        if !asset_cache.cached_backgrounds.contains_key(&background.image) {
            let handle = asset_server.load(&background.image);
            asset_cache.cached_backgrounds.insert(background.image.clone(), handle.clone());
            commands.spawn(LoadingAsset {
                asset_handle: handle,
                asset_path: background.image.clone(),
            });
        }
    }
    
    for (_, character) in &raven_story.story.characters {
        if !asset_cache.cached_characters.contains_key(&character.sprite) {
            let handle = asset_server.load(&character.sprite);
            asset_cache.cached_characters.insert(character.sprite.clone(), handle.clone());
            commands.spawn(LoadingAsset {
                asset_handle: handle,
                asset_path: character.sprite.clone(),
            });
        }
    }
}

fn check_asset_loading(
    mut commands: Commands,
    mut raven_story: ResMut<RavenStory>,
    loading_assets: Query<(Entity, &LoadingAsset)>,
    images: Res<Assets<Image>>,
) {
    let mut all_loaded = true;
    
    for (entity, loading_asset) in loading_assets.iter() {
        if images.get(&loading_asset.asset_handle).is_none() {
            all_loaded = false;
        } else {
            commands.entity(entity).despawn();
        }
    }
    
    if all_loaded && raven_story.waiting_for_asset_load {
        raven_story.waiting_for_asset_load = false;
    }
}