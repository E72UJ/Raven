// src/lib.rs
use bevy::prelude::*;

// 导出 AppState
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

// 最小的 MenuPlugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu);
    }
}

fn setup_menu() {
    println!("Menu setup called!");
}