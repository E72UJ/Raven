// src/menu/mod.rs
use bevy::prelude::*;
use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, handle_menu_input.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct MenuUI;

fn setup_menu(mut commands: Commands) {
    println("启动程序！！！");
    println!("Menu setup called 程序启动!");
    
    let menu_entity = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.2)),
        MenuUI,
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("Main Menu"),
            TextFont {
                font_size: 60.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // 说明文字
        parent.spawn((
            Text::new("Press SPACE to start game"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        
        // 退出说明
        parent.spawn((
            Text::new("Press ESC to exit"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    }).id();
    
    println!("Menu UI entity created: {:?}", menu_entity);
}

fn handle_menu_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Starting game...");
        next_state.set(AppState::InGame);
    }
}

fn cleanup_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuUI>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
        println!("Menu entity despawned: {:?}", entity);
    }
    println!("Menu cleaned up!");
}