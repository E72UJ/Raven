mod menu;

use bevy::prelude::*;
use bevy::input_focus::InputFocus;
use bevy::winit::WinitSettings;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    Settings,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings::desktop_app())
        .init_resource::<InputFocus>()
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(menu::MenuPlugin)
        .add_plugins(game::GamePlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}