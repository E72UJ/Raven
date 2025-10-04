use bevy::prelude::*;
pub mod dissolve;
pub mod menu;
pub use menu::MenuPlugin;
pub mod audio;
pub mod config;
pub mod position;
pub mod style;
pub mod toolbar;
pub mod typewriter;
pub mod url;
// 导出 GameScene，让外部可以使用
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameScene {
    #[default]
    Menu,
    Game,
    Settings,
    About,
    Help,
    Load,
    LoadButton,
    GameSettings,
}

pub use GameScene as AppState;
