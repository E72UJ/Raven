// examples/visual_novel_ui.rs
use bevy::prelude::*;

#[derive(Resource)]
struct FontAssets {
    main_font: Handle<Font>,
}

#[derive(Resource)]
struct VisualNovelState {
    auto_play_enabled: bool,
    auto_play_timer: f32,
    auto_play_delay: f32,
    current_dialogue_index: usize,
    text_display_timer: f32,
    chars_displayed: usize,
    text_speed: f32, // 每秒显示的字符数
}

impl Default for VisualNovelState {
    fn default() -> Self {
        Self {
            auto_play_enabled: false,
            auto_play_timer: 0.0,
            auto_play_delay: 3.0,
            current_dialogue_index: 0,
            text_display_timer: 0.0,
            chars_displayed: 0,
            text_speed: 30.0, // 每秒30个字符
        }
    }
}

#[derive(Resource)]
struct DialogueData {
    lines: Vec<DialogueLine>,
}

#[derive(Clone)]
struct DialogueLine {
    speaker: String,
    text: String,
    emotion: String,
}

impl Default for DialogueData {
    fn default() -> Self {
        Self {
            lines: vec![
                DialogueLine {
                    speaker: "系統".to_string(),
                    text: "歡迎來到《星際探險》視覺小說！".to_string(),
                    emotion: "neutral".to_string(),
                },
                DialogueLine {
                    speaker: "艾莉雅".to_string(),
                    text: "船長，我們即將抵達未知星球。感測器顯示這裡有生命跡象。".to_string(),
                    emotion: "excited".to_string(),
                },
                DialogueLine {
                    speaker: "船長".to_string(),
                    text: "很好，準備著陸程序。但要保持警惕，我們不知道會遇到什麼。".to_string(),
                    emotion: "serious".to_string(),
                },
                DialogueLine {
                    speaker: "艾莉雅".to_string(),
                    text: "明白！能量護盾已啟動，掃描設備運行正常。".to_string(),
                    emotion: "confident".to_string(),
                },
                DialogueLine {
                    speaker: "系統".to_string(),
                    text: "警告：檢測到未知能量波動。建議謹慎行動。".to_string(),
                    emotion: "warning".to_string(),
                },
                DialogueLine {
                    speaker: "船長".to_string(),
                    text: "看起來我們的冒險才剛剛開始...".to_string(),
                    emotion: "thoughtful".to_string(),
                },
                DialogueLine {
                    speaker: "艾莉雅".to_string(),
                    text: "無論遇到什麼，我們都會一起面對！".to_string(),
                    emotion: "determined".to_string(),
                },
                DialogueLine {
                    speaker: "系統".to_string(),
                    text: "故事未完待續... 感謝你的閱讀！".to_string(),
                    emotion: "neutral".to_string(),
                },
            ],
        }
    }
}

#[derive(Component)]
struct DialogueText;

#[derive(Component)]
struct SpeakerText;

#[derive(Component)]
struct AutoPlayIndicator;

#[derive(Component)]
struct ProgressIndicator;

#[derive(Event)]
struct AdvanceDialogueEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<VisualNovelState>()
        .init_resource::<DialogueData>()
        
        .add_event::<AdvanceDialogueEvent>()
        
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        
        .add_systems(Startup, (
            load_fonts,
            setup_ui,
        ).chain())
        
        .add_systems(Update, (
            handle_input,
            process_dialogue_events,
            update_dialogue_display,
        ).chain())
        
        .add_systems(FixedUpdate, (
            auto_play_timer_system,
            text_typewriter_system,
        ))
        
        .run();
}

fn load_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let main_font = asset_server.load("fonts/GenSenMaruGothicTW-Bold.ttf");
    
    commands.insert_resource(FontAssets {
        main_font,
    });
    
    println!("📝 正在載入字體: GenSenMaruGothicTW-Bold.ttf");
}

fn setup_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
) {
    // 相機
    commands.spawn(Camera2d);
    
    // 主UI容器
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2)),
    )).with_children(|parent| {
        // 狀態顯示區域
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        )).with_children(|parent| {
            // 自動播放指示器
            parent.spawn((
                Text::new("手動模式 | Space: 下一段 | A: 切換自動播放"),
                TextFont {
                    font: font_assets.main_font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                AutoPlayIndicator,
            ));
        });
        
        // 對話框容器
        parent.spawn((
            Node {
                width: Val::Percent(90.0),
                height: Val::Px(250.0),
                margin: UiRect::all(Val::Px(20.0)),
                padding: UiRect::all(Val::Px(25.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            BorderColor(Color::srgb(0.4, 0.4, 0.6)),
        )).with_children(|parent| {
            // 說話人名字
            parent.spawn((
                Text::new("系統"),
                TextFont {
                    font: font_assets.main_font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                SpeakerText,
            ));
            
            // 對話內容
            parent.spawn((
                Text::new("歡迎來到《星際探險》視覺小說！"),
                TextFont {
                    font: font_assets.main_font.clone(),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
                DialogueText,
            ));
            
            // 進度指示器
            parent.spawn((
                Text::new("1 / 8"),
                TextFont {
                    font: font_assets.main_font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    align_self: AlignSelf::FlexEnd,
                    ..default()
                },
                ProgressIndicator,
            ));
        });
    });
    
    println!("📖 視覺小說系統啟動 (Bevy 0.16 + 自訂字體)");
    println!("🎮 操作說明：");
    println!("   Space - 下一段對話");
    println!("   A - 切換自動播放模式");
    println!("🎨 字體：GenSenMaruGothicTW-Bold");
}

fn auto_play_timer_system(
    mut vn_state: ResMut<VisualNovelState>,
    fixed_time: Res<Time<Fixed>>,
    mut dialogue_events: EventWriter<AdvanceDialogueEvent>,
    dialogue_data: Res<DialogueData>,
) {
    if vn_state.auto_play_enabled {
        vn_state.auto_play_timer += fixed_time.delta_secs();
        
        if vn_state.auto_play_timer >= vn_state.auto_play_delay {
            vn_state.auto_play_timer = 0.0;
            
            // 只有當前文字完全顯示後才自動前進
            if vn_state.current_dialogue_index < dialogue_data.lines.len() {
                let current_text = &dialogue_data.lines[vn_state.current_dialogue_index].text;
                if vn_state.chars_displayed >= current_text.chars().count() {
                    dialogue_events.send(AdvanceDialogueEvent);
                }
            }
        }
    }
}

fn text_typewriter_system(
    mut vn_state: ResMut<VisualNovelState>,
    fixed_time: Res<Time<Fixed>>,
    dialogue_data: Res<DialogueData>,
) {
    if vn_state.current_dialogue_index < dialogue_data.lines.len() {
        let current_text = &dialogue_data.lines[vn_state.current_dialogue_index].text;
        let total_chars = current_text.chars().count();
        
        if vn_state.chars_displayed < total_chars {
            vn_state.text_display_timer += fixed_time.delta_secs();
            
            let chars_to_show = (vn_state.text_display_timer * vn_state.text_speed) as usize;
            vn_state.chars_displayed = chars_to_show.min(total_chars);
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut vn_state: ResMut<VisualNovelState>,
    mut dialogue_events: EventWriter<AdvanceDialogueEvent>,
    dialogue_data: Res<DialogueData>,
) {
    // 前進對話
    if keyboard.just_pressed(KeyCode::Space) {
        // 如果文字還在打字效果中，立即顯示完整文字
        if vn_state.current_dialogue_index < dialogue_data.lines.len() {
            let current_text = &dialogue_data.lines[vn_state.current_dialogue_index].text;
            let total_chars = current_text.chars().count();
            
            if vn_state.chars_displayed < total_chars {
                vn_state.chars_displayed = total_chars;
                return;
            }
        }
        
        // 否則前進到下一段對話
        vn_state.auto_play_timer = 0.0;
        dialogue_events.send(AdvanceDialogueEvent);
    }
    
    // 切換自動播放
    if keyboard.just_pressed(KeyCode::KeyA) {
        vn_state.auto_play_enabled = !vn_state.auto_play_enabled;
        vn_state.auto_play_timer = 0.0;
    }
}

fn process_dialogue_events(
    mut dialogue_events: EventReader<AdvanceDialogueEvent>,
    mut vn_state: ResMut<VisualNovelState>,
    dialogue_data: Res<DialogueData>,
) {
    for _event in dialogue_events.read() {
        if vn_state.current_dialogue_index < dialogue_data.lines.len() - 1 {
            vn_state.current_dialogue_index += 1;
            vn_state.chars_displayed = 0;
            vn_state.text_display_timer = 0.0;
        } else {
            // 對話結束
            vn_state.auto_play_enabled = false;
            println!("🎉 故事結束！");
        }
    }
}

fn update_dialogue_display(
    vn_state: Res<VisualNovelState>,
    dialogue_data: Res<DialogueData>,
    mut speaker_query: Query<(&mut Text, &mut TextColor), (With<SpeakerText>, Without<DialogueText>, Without<AutoPlayIndicator>, Without<ProgressIndicator>)>,
    mut dialogue_query: Query<&mut Text, (With<DialogueText>, Without<SpeakerText>, Without<AutoPlayIndicator>, Without<ProgressIndicator>)>,
    mut auto_play_query: Query<(&mut Text, &mut TextColor), (With<AutoPlayIndicator>, Without<SpeakerText>, Without<DialogueText>, Without<ProgressIndicator>)>,
    mut progress_query: Query<&mut Text, (With<ProgressIndicator>, Without<SpeakerText>, Without<DialogueText>, Without<AutoPlayIndicator>)>,
) {
    if vn_state.current_dialogue_index < dialogue_data.lines.len() {
        let current_line = &dialogue_data.lines[vn_state.current_dialogue_index];
        
        // 更新說話人
        if let Ok((mut speaker_text, mut speaker_color)) = speaker_query.single_mut() {
            **speaker_text = current_line.speaker.clone();
            
            // 根據情感設置顏色
            speaker_color.0 = match current_line.emotion.as_str() {
                "excited" => Color::srgb(1.0, 0.8, 0.2),     // 興奮 - 金黃色
                "serious" => Color::srgb(0.8, 0.8, 1.0),     // 嚴肅 - 淡藍色
                "confident" => Color::srgb(0.2, 1.0, 0.4),   // 自信 - 綠色
                "warning" => Color::srgb(1.0, 0.3, 0.3),     // 警告 - 紅色
                "thoughtful" => Color::srgb(0.8, 0.6, 1.0),  // 沉思 - 紫色
                "determined" => Color::srgb(1.0, 0.6, 0.2),  // 堅決 - 橙色
                _ => Color::srgb(0.9, 0.9, 0.9),             // 預設 - 白色
            };
        }
        
        // 更新對話內容（打字機效果）
        if let Ok(mut dialogue_text) = dialogue_query.single_mut() {
            let full_text = &current_line.text;
            let displayed_text: String = full_text.chars()
                .take(vn_state.chars_displayed)
                .collect();
            
            **dialogue_text = displayed_text;
        }
        
        // 更新進度指示器
        if let Ok(mut progress_text) = progress_query.single_mut() {
            **progress_text = format!("{} / {}", 
                vn_state.current_dialogue_index + 1, 
                dialogue_data.lines.len());
        }
    }
    
    // 更新自動播放指示器
    if let Ok((mut auto_play_text, mut auto_play_color)) = auto_play_query.single_mut() {
        if vn_state.auto_play_enabled {
            let remaining = (vn_state.auto_play_delay - vn_state.auto_play_timer).max(0.0);
            **auto_play_text = format!(
                "自動播放中 ({:.1}s) | Space: 下一段 | A: 關閉自動播放", 
                remaining
            );
            auto_play_color.0 = Color::srgb(0.2, 1.0, 0.4);
        } else {
            **auto_play_text = "手動模式 | Space: 下一段 | A: 切換自動播放".to_string();
            auto_play_color.0 = Color::WHITE;
        }
    }
}
