use bevy::prelude::*;
use bevy::window::{Window, WindowResolution, EnabledButtons};

pub mod window {
    use super::*;
    
    pub const TITLE: &str = "Raven Visual Novel Engine";
    pub const WIDTH: u32 = 1400;    
    pub const HEIGHT: u32 = 770;    
    pub const RESIZABLE: bool = true;
    pub const MAXIMIZE: bool = true;
    pub const MINIMIZE: bool = true;
    pub const CLOSE: bool = true;
    
    pub fn build() -> Window {
        Window {
            title: TITLE.to_string(),
            resolution: WindowResolution::new(WIDTH, HEIGHT),
            resizable: RESIZABLE,
            enabled_buttons: EnabledButtons {
                maximize: MAXIMIZE,
                minimize: MINIMIZE,
                close: CLOSE,
            },
            ..default()
        }
    }
}

// 游戏配置
pub mod game {
    pub const VERSION: &str = "0.0.1";
    pub const AUTO_SAVE_INTERVAL: u64 = 300;
}

pub mod ui {
    pub const TEXT_SPEED: f32 = 1.0;
    pub const FONT_SIZE_SPEAKER: f32 = 24.0;
    pub const FONT_SIZE_DIALOGUE: f32 = 18.0;
    pub const DIALOGUE_BOX_HEIGHT: f32 = 200.0;
    pub const DIALOGUE_BOX_WIDTH_PERCENT: f32 = 90.0;
}


pub mod audio {
    pub const MASTER_VOLUME: f32 = 1.0;
    pub const BGM_VOLUME: f32 = 0.8;
    pub const SFX_VOLUME: f32 = 0.9;
    pub const VOICE_VOLUME: f32 = 1.0;
}


pub mod debug {
    pub const SHOW_FPS: bool = false;
    pub const SHOW_DIAGNOSTICS: bool = false;
    pub const LOG_LEVEL: &str = "info";
}
