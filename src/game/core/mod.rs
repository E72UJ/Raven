use bevy::prelude::*;
use crate::{
    GameState,
    LabelMap,
    MainConfig,
    PortraitAssets,
    load_dialogues,
    play_sound,
    ClickSound,
    BackClickSound,
};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_camera)
            .add_startup_system(setup_ui)
            .add_system(handle_input)
            .add_system(update_portrait);
    }
}

fn setup_camera(mut commands: Commands, config: Res<MainConfig>) {
    commands.spawn(Camera2d);

    // 初始化游戏状态
    let dialogues = load_dialogues(&config);
    let mut label_map = HashMap::new();
    for (index, dialogue) in dialogues.iter().enumerate() {
        if let Some(label) = dialogue.label.as_ref() {
            label_map.insert(label.clone(), index);
        }
    }

    commands.insert_resource(GameState {
        current_line: 0,
        dialogues,
        can_go_back: false,
        jump_label: None,
        in_branch_selection: false,
    });
    commands.insert_resource(LabelMap(label_map));
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MainConfig>) {
    // 创建用户界面元素,如点击区域、对话框等
}

fn handle_input(
    mut interaction_query: Query<(&Interaction, &Name), (Changed<Interaction>, With<Node>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
    click_sound: Res<ClickSound>,
    back_sound: Res<BackClickSound>,
    label_map: Res<LabelMap>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
    mut commands: Commands, // 添加 mut 关键字
) {
    // ESC键始终可用
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // 数字键快速跳转（始终可用）
    for key in keys.get_just_pressed() {
        match key {
            KeyCode::Digit0 => game_state.current_line = 0,
            KeyCode::Digit1 => game_state.current_line = 1,
            KeyCode::Digit2 => game_state.current_line = 2,
            _ => {}
        }
    }

    // 返回上一页（始终可用）
    let back_pressed = keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::ArrowLeft);
    if back_pressed && game_state.can_go_back && game_state.current_line > 0 {
        game_state.current_line -= 1;
        play_sound(&back_sound.0, commands.reborrow());
        if game_state.current_line == 0 {
            game_state.can_go_back = false;
        }
    }

    // 如果在分支选择状态，禁用前进操作
    if game_state.in_branch_selection {
        return;
    }

    // 检测前进输入（键盘 + 鼠标 + 点击区域）
    let keyboard_click = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter);
    let mouse_click = mouse.just_pressed(MouseButton::Left);
    
    // 检查点击区域
    let mut click_area_pressed = false;
    for (interaction, name) in &interaction_query {
        if *interaction == Interaction::Pressed && name.as_str() == "click_area" {
            click_area_pressed = true;
            println!("点击了透明区域");
            break;
        }
    }

    // 统一处理前进逻辑
    let should_advance = keyboard_click || mouse_click || click_area_pressed;
    
    if should_advance && game_state.current_line < game_state.dialogues.len() {
        let current_dialogue = &game_state.dialogues[game_state.current_line];
        
        // 检查是否有跳转指令
        if let Some(jump_label) = &current_dialogue.jump {
            game_state.jump_label = Some(jump_label.clone());
        } else {
            // 没有跳转指令则前进到下一行
            game_state.current_line += 1;
        }
        
        game_state.can_go_back = true;
        play_sound(&back_sound.0, commands.reborrow());
    }
}
fn update_portrait(
    game_state: Res<GameState>,
    portraits: Res<PortraitAssets>,
    mut query: Query<(&mut Sprite, &mut Name, &mut Visibility)>,
) {
    // 更新角色立绘的显示
}