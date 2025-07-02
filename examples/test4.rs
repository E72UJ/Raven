// examples/test4.rs
use bevy::prelude::*;
use Raven::{AppState, MenuPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_plugins(MenuPlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (handle_escape_key, debug_state, check_entities))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    println!("Camera spawned!");
}

fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn debug_state(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        println!("Current state: {:?}", current_state.get());
        next_state.set(AppState::Menu);
    }
}

fn check_entities(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, With<Node>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        println!("UI entities count: {}", query.iter().count());
    }
}