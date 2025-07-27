// src/lib.rs
use bevy::prelude::*;
pub mod dissolve;
pub mod menu;
pub use menu::MenuPlugin;
pub mod typewriter;
pub mod style;
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
    LoadButton
}

// 如果你想要一个叫 AppState 的别名，可以这样做：
pub use GameScene as AppState;