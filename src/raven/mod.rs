// src/raven/mod.rs
pub mod bevy_integration;

pub mod script {  
    use std::collections::HashMap;
    use crate::raven::character::Character;
    use crate::raven::scene::Scene;
    use crate::raven::background::Background;

    #[derive(Debug, Clone)]
    pub struct Script {  
        pub characters: HashMap<String, Character>,
        pub scenes: HashMap<String, Scene>,
        pub backgrounds: HashMap<String, Background>,
        pub start_scene: Option<String>,
    }

    impl Script {  
        pub fn new() -> Self {
            Self {
                characters: HashMap::new(),
                scenes: HashMap::new(),
                backgrounds: HashMap::new(),
                start_scene: None,
            }
        }

        pub fn add_character(&mut self, id: String, character: Character) {
            self.characters.insert(id, character);
        }

        pub fn add_scene(&mut self, id: String, scene: Scene) {
            if self.start_scene.is_none() {
                self.start_scene = Some(id.clone());
            }
            self.scenes.insert(id, scene);
        }

        pub fn add_background(&mut self, id: String, background: Background) {
            self.backgrounds.insert(id, background);
        }

        pub fn get_character(&self, id: &str) -> Option<&Character> {
            self.characters.get(id)
        }

        pub fn get_scene(&self, id: &str) -> Option<&Scene> {
            self.scenes.get(id)
        }

        pub fn get_background(&self, id: &str) -> Option<&Background> {
            self.backgrounds.get(id)
        }
    }
}

pub mod character {
    #[derive(Debug, Clone)]
    pub struct Character {
        pub name: String,
        pub sprite: String,
        pub color: Option<String>,
    }

    impl Character {
        pub fn new(name: String, sprite: String) -> Self {
            Self {
                name,
                sprite,
                color: None,
            }
        }

        pub fn with_color(mut self, color: String) -> Self {
            self.color = Some(color);
            self
        }
    }
}

pub mod scene {
    #[derive(Debug, Clone)]
    pub struct Scene {
        pub commands: Vec<SceneCommand>,
    }

    #[derive(Debug, Clone)]
    pub enum SceneCommand {
        PlayMusic {
            file: String,
        },
        ShowBackground {
            background: String,
        },
        ShowCharacter {
            character: String,
            emotion: Option<String>,
        },
        HideCharacter {
            character: String,
        },
        Dialogue {
            speaker: String,
            text: String,
        },
        PlayerThinks {
            text: String,
        },
        PlayerSays {
            text: String,
        },
        ShowChoices {
            choices: Vec<Choice>,
        },
        Jump {
            scene: String,
        },
        EndWith {
            ending: String,
        },
        ExitGame, // 新添加这一行
    }

    #[derive(Debug, Clone)]
    pub struct Choice {
        pub text: String,
        pub scene: String,
    }

    impl Scene {
        pub fn new() -> Self {
            Self {
                commands: Vec::new(),
            }
        }

        pub fn add_command(&mut self, command: SceneCommand) {
            self.commands.push(command);
        }
    }

    impl Choice {
        pub fn new(text: String, scene: String) -> Self {
            Self { text, scene }
        }
    }
}

pub mod background {
    #[derive(Debug, Clone)]
    pub struct Background {
        pub image: String,
        pub music: Option<String>,
    }

    impl Background {
        pub fn new(image: String) -> Self {
            Self {
                image,
                music: None,
            }
        }

        pub fn with_music(mut self, music: String) -> Self {
            self.music = Some(music);
            self
        }
    }
}

pub mod game {
    use crate::raven::script::Script;
    use std::sync::{Mutex, OnceLock};

    #[derive(Debug, Clone, PartialEq)]
    pub enum GameResult {
        Playing,
        Ending(String),
        Quit,
        Error(String),
    }

    static GAME_STATE: OnceLock<Mutex<GameState>> = OnceLock::new();

    #[derive(Debug)]
    struct GameState {
        script: Option<Script>,
        current_scene: Option<String>,
        result: GameResult,
    }

    impl GameState {
        fn new() -> Self {
            Self {
                script: None,
                current_scene: None,
                result: GameResult::Playing,
            }
        }
    }

    fn init_game_state() -> &'static Mutex<GameState> {
        GAME_STATE.get_or_init(|| Mutex::new(GameState::new()))
    }

    pub fn run_raven_game_with_story(script: Option<Script>) { 
        let game_state = init_game_state();
        
        {
            let mut state = game_state.lock().unwrap();
            state.current_scene = script.as_ref().and_then(|s| s.start_scene.clone());
            state.script = script;  
            state.result = GameResult::Playing;
        }
        
        println!("Raven 游戏引擎已启动！");
        
        let state = game_state.lock().unwrap();
        if let Some(script) = &state.script {  
            println!("脚本已加载，包含 {} 个角色，{} 个场景，{} 个背景", 
                script.characters.len(), 
                script.scenes.len(), 
                script.backgrounds.len()
            );
            
            if let Some(start_scene_id) = &script.start_scene {
                if let Some(scene) = script.get_scene(start_scene_id) {
                    println!("开始执行场景: {}", start_scene_id);
                    for command in &scene.commands {
                        execute_command(command, script);
                    }
                }
            }
        }
    }

    fn execute_command(command: &crate::raven::scene::SceneCommand, script: &Script) {
        use crate::raven::scene::SceneCommand;
        
        match command {
            SceneCommand::PlayMusic { file } => {
                println!("🎵 播放音乐: {}", file);
            },
            SceneCommand::ShowBackground { background } => {
                if let Some(bg) = script.get_background(background) {
                    println!("🖼️ 显示背景: {} ({})", background, bg.image);
                }
            },
            SceneCommand::ShowCharacter { character, emotion } => {
                if let Some(char) = script.get_character(character) {
                    let emotion_text = emotion.as_ref().map(|e| format!(" [{}]", e)).unwrap_or_default();
                    println!("👤 显示角色: {}{} ({})", char.name, emotion_text, char.sprite);
                }
            },
            SceneCommand::HideCharacter { character } => {
                println!("👻 隐藏角色: {}", character);
            },
            SceneCommand::Dialogue { speaker, text } => {
                if let Some(char) = script.get_character(speaker) {
                    println!("💬 {}: \"{}\"", char.name, text);
                } else {
                    println!("💬 {}: \"{}\"", speaker, text);
                }
            },
            SceneCommand::PlayerThinks { text } => {
                println!("💭 (内心想法): {}", text);
            },
            SceneCommand::PlayerSays { text } => {
                println!("🗣️ 玩家: \"{}\"", text);
            },
            SceneCommand::ShowChoices { choices } => {
                println!("🔘 选择:");
                for (i, choice) in choices.iter().enumerate() {
                    println!("  {}. {} -> {}", i + 1, choice.text, choice.scene);
                }
            },
            SceneCommand::Jump { scene } => {
                println!("↗️ 跳转到场景: {}", scene);
            },
            SceneCommand::EndWith { ending } => {
                println!("🏁 游戏结束: {}", ending);
                set_game_ending(ending.clone());
            },
            SceneCommand::ExitGame => {
                println!("🚪 退出游戏");
                end_raven_game();
            },
        }
    }

    pub fn get_game_result() -> GameResult {
        let game_state = init_game_state();
        let state = game_state.lock().unwrap();
        state.result.clone()
    }

    pub fn end_raven_game() {
        let game_state = init_game_state();
        let mut state = game_state.lock().unwrap();
        println!("当前游戏状态: {:?}", state.result);
        
        if state.result == GameResult::Playing {
            state.result = GameResult::Quit;
            println!("游戏状态已设置为 Quit");
        } else {
            println!("游戏已经结束，当前状态: {:?}", state.result);
        }
        println!("Raven 游戏引擎已关闭");
    }

    pub fn set_game_ending(ending: String) {
        let game_state = init_game_state();
        let mut state = game_state.lock().unwrap();
        state.result = GameResult::Ending(ending);
    }

    pub fn get_current_story() -> Option<Script> {
        let game_state = init_game_state();
        let state = game_state.lock().unwrap();
        state.script.clone()
    }
}

#[macro_export]
macro_rules! Rvn {
    ($($item:tt)*) => {{
        let mut script = $crate::raven::script::Script::new();
        $crate::parse_story_items!(script, $($item)*);
        script
    }};
}

#[macro_export]
macro_rules! parse_story_items {
    ($script:ident,) => {};
    
    ($script:ident, character $char_id:ident { $($char_content:tt)* } $($rest:tt)*) => {
        let character = $crate::parse_character!($($char_content)*);
        $script.add_character(stringify!($char_id).to_string(), character);
        $crate::parse_story_items!($script, $($rest)*);
    };
    
    ($script:ident, background $bg_id:ident { $($bg_content:tt)* } $($rest:tt)*) => {
        let background = $crate::parse_background!($($bg_content)*);
        $script.add_background(stringify!($bg_id).to_string(), background);
        $crate::parse_story_items!($script, $($rest)*);
    };
    
    ($script:ident, scene $scene_id:ident { $($scene_content:tt)* } $($rest:tt)*) => {
        let scene = $crate::parse_scene!($($scene_content)*);
        $script.add_scene(stringify!($scene_id).to_string(), scene);
        $crate::parse_story_items!($script, $($rest)*);
    };
}

#[macro_export]
macro_rules! parse_character {
    ($($content:tt)*) => {{
        let mut name = String::new();
        let mut sprite = String::new();
        let mut color: Option<String> = None;
        $crate::parse_character_fields!(name, sprite, color, $($content)*);
        
        let mut character = $crate::raven::character::Character::new(name, sprite);
        if let Some(c) = color {
            character = character.with_color(c);
        }
        character
    }};
}

#[macro_export]
macro_rules! parse_character_fields {
    ($name:ident, $sprite:ident, $color:ident,) => {};
    
    ($name:ident, $sprite:ident, $color:ident, name = $value:expr; $($rest:tt)*) => {
        $name = $value.to_string();
        $crate::parse_character_fields!($name, $sprite, $color, $($rest)*);
    };
    
    ($name:ident, $sprite:ident, $color:ident, sprite = $value:expr; $($rest:tt)*) => {
        $sprite = $value.to_string();
        $crate::parse_character_fields!($name, $sprite, $color, $($rest)*);
    };
    
    ($name:ident, $sprite:ident, $color:ident, color = $value:expr; $($rest:tt)*) => {
        $color = Some($value.to_string());
        $crate::parse_character_fields!($name, $sprite, $color, $($rest)*);
    };
}

#[macro_export]
macro_rules! parse_background {
    (image = $image:expr; $($rest:tt)*) => {{
        let mut background = $crate::raven::background::Background::new($image.to_string());
        $crate::parse_background_fields!(background, $($rest)*);
        background
    }};
}

#[macro_export]
macro_rules! parse_background_fields {
    ($bg:ident,) => {};
    
    ($bg:ident, music = $value:expr; $($rest:tt)*) => {
        $bg = $bg.with_music($value.to_string());
        $crate::parse_background_fields!($bg, $($rest)*);
    };
}

#[macro_export]
macro_rules! parse_scene {
    ($($content:tt)*) => {{
        let mut scene = $crate::raven::scene::Scene::new();
        $crate::parse_scene_commands!(scene, $($content)*);
        scene
    }};
}

#[macro_export]
macro_rules! parse_scene_commands {
    ($scene:ident,) => {};
    
    ($scene:ident, play music $file:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::PlayMusic {
            file: $file.to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, show background $bg:ident $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::ShowBackground {
            background: stringify!($bg).to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, show character $char:ident $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::ShowCharacter {
            character: stringify!($char).to_string(),
            emotion: None,
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, show character $char:ident as $emotion:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::ShowCharacter {
            character: stringify!($char).to_string(),
            emotion: Some($emotion.to_string()),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, $char:ident says $text:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::Dialogue {
            speaker: stringify!($char).to_string(),
            text: $text.to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, player thinks $text:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::PlayerThinks {
            text: $text.to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, player says $text:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::PlayerSays {
            text: $text.to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, show choices { $($choice_content:tt)* } $($rest:tt)*) => {
        let mut choices = Vec::new();
        $crate::parse_choices!(choices, $($choice_content)*);
        $scene.add_command($crate::raven::scene::SceneCommand::ShowChoices { choices });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, jump to $target:ident $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::Jump {
            scene: stringify!($target).to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
    
    ($scene:ident, end with $ending:literal $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::EndWith {
            ending: $ending.to_string(),
        });
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
   ($scene:ident, exit game $($rest:tt)*) => {
        $scene.add_command($crate::raven::scene::SceneCommand::ExitGame);
        $crate::parse_scene_commands!($scene, $($rest)*);
    };
}

#[macro_export]
macro_rules! parse_choices {
    ($choices:ident,) => {};
    
    ($choices:ident, $text:literal -> $scene:ident, $($rest:tt)*) => {
        $choices.push($crate::raven::scene::Choice::new(
            $text.to_string(),
            stringify!($scene).to_string()
        ));
        $crate::parse_choices!($choices, $($rest)*);
    };
    
    ($choices:ident, $text:literal -> $scene:ident $($rest:tt)*) => {
        $choices.push($crate::raven::scene::Choice::new(
            $text.to_string(),
            stringify!($scene).to_string()
        ));
        $crate::parse_choices!($choices, $($rest)*);
    };
}

pub mod prelude {
    pub use crate::raven::script::*;
    pub use crate::raven::character::*;
    pub use crate::raven::scene::*;
    pub use crate::raven::background::*;
    pub use crate::raven::game::*;
    pub use crate::raven::bevy_integration::run_raven_game;  

    pub use crate::Rvn;
    pub use crate::parse_story_items;
    pub use crate::parse_character;
    pub use crate::parse_character_fields;
    pub use crate::parse_background;
    pub use crate::parse_background_fields;
    pub use crate::parse_scene;
    pub use crate::parse_scene_commands;
    pub use crate::parse_choices;
}

pub use script::Script;
pub use character::Character;
pub use scene::{Scene, SceneCommand, Choice};
pub use background::Background;
pub use game::{GameResult, run_raven_game_with_story, get_game_result, set_game_ending};
pub use bevy_integration::{run_raven_game, end_raven_game};
