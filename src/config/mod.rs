use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Resource, Deserialize, Serialize, Clone)]
pub struct MainConfig {
    pub title: String,
    pub assets: AssetsConfig,
    pub settings: SettingsConfig,
    pub global_variables: GlobalVariables,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AssetsConfig {
    pub characters: HashMap<String, String>,
    pub backgrounds: HashMap<String, String>,
    pub audio: AudioConfig,
    pub videos: HashMap<String, String>,
    pub swf: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AudioConfig {
    pub bgm: HashMap<String, String>,
    pub sfx: HashMap<String, String>,
    pub click_sound: String,
    pub backclick_sound: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsConfig {
    pub initial_scene: String,
    pub text_speed: u32,
    pub auto_save: bool,
    pub resolution: [u32; 2],
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GlobalVariables {
    pub player_name: String,
    pub affection_points: i32,
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            title: "Raven Engine V0.1".to_string(),
            assets: AssetsConfig {
                characters: HashMap::new(),
                backgrounds: HashMap::new(),
                audio: AudioConfig {
                    bgm: HashMap::new(),
                    sfx: HashMap::new(),
                    click_sound: "typing2.ogg".to_string(),
                    backclick_sound: "button.ogg".to_string(),
                },
                videos: HashMap::new(),
                swf: HashMap::new(),
            },
            settings: SettingsConfig {
                initial_scene: "intro".to_string(),
                text_speed: 50,
                auto_save: true,
                resolution: [1200, 660],
            },
            global_variables: GlobalVariables {
                player_name: "主角".to_string(),
                affection_points: 0,
            },
        }
    }
}

impl MainConfig {
    pub fn load() -> Self {
        load_main_config()
    }
    
    // 便于访问的辅助方法
    pub fn get_window_size(&self) -> (f32, f32) {
        (self.settings.resolution[0] as f32, self.settings.resolution[1] as f32)
    }
    
    pub fn get_character_path(&self, character: &str) -> Option<&String> {
        self.assets.characters.get(character)
    }
    
    pub fn get_background_path(&self, background: &str) -> Option<&String> {
        self.assets.backgrounds.get(background)
    }
}

pub fn load_main_config() -> MainConfig {
    let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let yaml_path = exe_dir.join("assets/main.yaml");
    
    println!("配置文件路径: {:?}", yaml_path);
    
    if !yaml_path.exists() {
        println!("配置文件不存在，使用默认配置");
        return MainConfig::default();
    }
    
    let yaml_str = match fs::read_to_string(&yaml_path) {
        Ok(content) => content,
        Err(e) => {
            println!("读取配置文件失败: {}, 使用默认配置", e);
            return MainConfig::default();
        }
    };
    
    match serde_yaml::from_str(&yaml_str) {
        Ok(config) => config,
        Err(e) => {
            println!("YAML解析失败: {}, 使用默认配置", e);
            MainConfig::default()
        }
    }
}